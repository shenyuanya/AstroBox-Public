package com.astralsight.astrobox.plugin.btclassic_spp

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothSocket
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Context.RECEIVER_NOT_EXPORTED
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.webkit.WebView
import androidx.annotation.RequiresPermission
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.actor
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.util.*
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

@OptIn(ObsoleteCoroutinesApi::class)
class BTSpp(private val context: Context, private val webView: WebView) {

    private val SPP_PREFIX = "00001101"          // 0x1101u → SPP

    private val REQUIRED_PERMISSIONS = arrayOf(
        Manifest.permission.BLUETOOTH,
        Manifest.permission.BLUETOOTH_ADMIN,
        Manifest.permission.BLUETOOTH_SCAN,
        Manifest.permission.BLUETOOTH_CONNECT
    )
    private val PERMISSION_REQUEST_CODE = 1001

    private val adapter: BluetoothAdapter? = BluetoothAdapter.getDefaultAdapter()
    private val scannedDevices = mutableListOf<BluetoothDevice>()

    private var socket: BluetoothSocket? = null
    private var inStream: InputStream? = null
    private var outStream: OutputStream? = null
    private var connectedDevice: BluetoothDevice? = null

    private val sendScope = CoroutineScope(SupervisorJob() + Dispatchers.IO);
    private var sendActor: SendChannel<ByteArray>? = null;

    private var readThread: Thread? = null
    private val uiHandler = Handler(Looper.getMainLooper())

    interface DataListener {
        fun onDataReceived(data: ByteArray)
        fun onError(e: IOException)
    }
    private var dataListener: DataListener? = null
    private var onConnectedCallback: (() -> Unit)? = null

    fun getScannedDevices(): List<BluetoothDevice> = scannedDevices.toList()
    fun getConnectedDeviceInfo(): BluetoothDevice? = connectedDevice
    fun setDataListener(listener: DataListener) { dataListener = listener }

    @SuppressLint("MissingPermission")
    fun initPermissions() {
        if (context is Activity) {
            ActivityCompat.requestPermissions(
                context,
                REQUIRED_PERMISSIONS,
                PERMISSION_REQUEST_CODE
            )
        }
        else {
            throw IllegalStateException("需要传入 Activity 作为 context，才能申请运行时权限")
        }
    }

    private suspend fun webViewLog(content: String){
        withContext(Dispatchers.Main){
            webView.evaluateJavascript("console.log('$content')", null)
        }
    }

    @SuppressLint("MissingPermission")
    @RequiresPermission(allOf = [Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_ADMIN])
    fun startScan() {
        scannedDevices.clear()
        adapter?.let { bt ->
            if (bt.isDiscovering) bt.cancelDiscovery()
            context.registerReceiver(scanReceiver, IntentFilter(BluetoothDevice.ACTION_FOUND))
            bt.startDiscovery()
        }
    }

    @SuppressLint("MissingPermission")
    fun stopScan() {
        adapter?.cancelDiscovery()
        try { context.unregisterReceiver(scanReceiver) } catch (_: IllegalArgumentException) {}
    }

    @RequiresPermission(Manifest.permission.BLUETOOTH_CONNECT)
    suspend fun connect(
        context: Context,
        address: String
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            /* 1️⃣ 运行时权限 */
            if (ContextCompat.checkSelfPermission(
                    context, Manifest.permission.BLUETOOTH_CONNECT
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                ActivityCompat.requestPermissions(
                    context as Activity,
                    arrayOf(Manifest.permission.BLUETOOTH_CONNECT),
                    0
                )
                return@withContext false
            }

            val adapter = (context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter
                ?: return@withContext false
            val dev = adapter.getRemoteDevice(address) ?: return@withContext false
            if (adapter.isDiscovering) adapter.cancelDiscovery()

            if (dev.bondState != BluetoothDevice.BOND_BONDED) {
                webViewLog("Kotlin: start bonding…")
                try {
                    dev.awaitBonded(context)
                    webViewLog("Kotlin: Bond successful!")
                } catch (e: Exception) {
                    webViewLog("Bond failed: ${e.message}")
                    return@withContext false
                }
            }

            val sock = tryChannel(dev, 5, 3_000)
                ?: tryChannel(dev, 1, 2_000)
                ?: trySdpUuid(dev)
                ?: return@withContext false

            webViewLog("Kotlin: Back from try process")

            socket = sock
            inStream = sock.inputStream
            outStream = sock.outputStream
            connectedDevice = dev

            sendActor = sendScope.actor(capacity = Channel.UNLIMITED) {
                for (payload in channel) {
                    try {
                        outStream?.write(payload)
                        outStream?.flush()
                    } catch (e: IOException) {
                        uiHandler.post { dataListener?.onError(e) }
                        break
                    }
                }
            }

            webViewLog("Kotlin: exec onConnectedCallback")

            onConnectedCallback?.let { cb ->
                webViewLog("Kotlin: exec cb...")
                uiHandler.post { cb() }
                onConnectedCallback = null
            }
            true
        } catch (e: Exception) {
            webViewLog("Connect failed: ${e.message}")
            false
        }
    }

    @RequiresPermission(Manifest.permission.BLUETOOTH_CONNECT)
    suspend fun BluetoothDevice.awaitBonded(
        context: Context,
        timeoutMs: Long = 15_000L
    ) {
        if (bondState == BluetoothDevice.BOND_BONDED) return           // 已配对

        if (!createBond()) throw IOException("createBond() failed")    // 触发配对

        // ① 把 withTimeout 放在最外层，它本身就是挂起函数
        withTimeout(timeoutMs) {
            suspendCancellableCoroutine<Unit> { cont ->
                val filter = IntentFilter(BluetoothDevice.ACTION_BOND_STATE_CHANGED)
                val receiver = object : BroadcastReceiver() {
                    @SuppressLint("MissingPermission")
                    override fun onReceive(ctx: Context?, intent: Intent?) {
                        val dev = intent?.getParcelableExtra<BluetoothDevice>(
                            BluetoothDevice.EXTRA_DEVICE
                        )
                        if (dev?.address != address) return            // 只关心目标设备
                        if (dev != null) {
                            when (dev.bondState) {
                                BluetoothDevice.BOND_BONDED -> {
                                    ctx?.unregisterReceiver(this)
                                    if (cont.isActive) cont.resume(Unit)
                                }

                                BluetoothDevice.BOND_NONE -> {
                                    ctx?.unregisterReceiver(this)
                                    if (cont.isActive)
                                        cont.resumeWithException(IOException("Bonding failed"))
                                }
                            }
                        }
                    }
                }
                context.registerReceiver(receiver, filter)

                cont.invokeOnCancellation { context.unregisterReceiver(receiver) }
            }
        }
    }

    @SuppressLint("MissingPermission")
    private suspend fun trySdpUuid(dev: BluetoothDevice): BluetoothSocket? {
        if (!dev.fetchUuidsWithSdp()) return null

        repeat(20) {
            /* ★ 只看 uuid 字符串是否以 00001101 开头 ★ */
            dev.uuids
                ?.firstOrNull { it.uuid.toString().startsWith(SPP_PREFIX, ignoreCase = true) }
                ?.let { parcel ->
                    return runCatching {
                        webViewLog("Kotlin: trySdpUuid(uuid=${parcel.uuid})")
                        val sock = dev.createRfcommSocketToServiceRecord(parcel.uuid) // secure
                        withTimeout(6_000) { sock.connect() }
                        sock
                    }.getOrNull()
                }
            delay(100)
        }
        return null
    }

    private suspend fun tryChannel(
        dev: BluetoothDevice,
        ch: Int,
        timeoutMs: Long
    ): BluetoothSocket? = runCatching {
        webViewLog("Kotlin: tryChannel(ch=$ch)")
        val m = dev.javaClass.getMethod("createRfcommSocket", Int::class.javaPrimitiveType)
        val sock = m.invoke(dev, ch) as BluetoothSocket
        webViewLog("Kotlin: connecting to sock…")
        withTimeout(timeoutMs) { sock.connect() }
        webViewLog("Kotlin: connected!…")
        sock
    }.getOrNull()

    fun onConnected(cb: () -> Unit) {
        if (connectedDevice != null) {
            uiHandler.post { cb() }
        } else {
            onConnectedCallback = cb
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    fun send(data: ByteArray): Boolean {
        val actor = sendActor
        return if (actor != null && !actor.isClosedForSend) {
            actor.trySend(data).isSuccess
        } else {
            false
        }
    }

    fun startSubscription() {
        if (inStream == null || readThread != null) return
        readThread = Thread {
            val buf = ByteArray(1024)
            try {
                while (!Thread.currentThread().isInterrupted) {
                    val len = inStream?.read(buf) ?: break
                    if (len <= 0) break   // -1 结束，0 继续
                    val bytes = buf.copyOf(len)
                    uiHandler.post { dataListener?.onDataReceived(bytes) }
                }
            } catch (e: IOException) {
                uiHandler.post { dataListener?.onError(e) }
            } finally {
                disconnect()
            }
        }.also { it.start() }
    }

    @SuppressLint("MissingPermission")
    fun disconnect() {
        readThread?.interrupt(); readThread = null

        sendActor?.close()
        sendActor = null
        sendScope.coroutineContext.cancelChildren()

        try { inStream?.close() } catch (_: Exception) {}
        try { outStream?.close() } catch (_: Exception) {}
        try { socket?.close() } catch (_: Exception) {}
        inStream = null; outStream = null; socket = null; connectedDevice = null
    }

    private val scanReceiver = object : BroadcastReceiver() {
        override fun onReceive(ctx: Context?, intent: Intent) {
            if (intent.action == BluetoothDevice.ACTION_FOUND) {
                (intent.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE) as? BluetoothDevice)
                    ?.takeIf { !scannedDevices.contains(it) }
                    ?.let(scannedDevices::add)
            }
        }
    }
}
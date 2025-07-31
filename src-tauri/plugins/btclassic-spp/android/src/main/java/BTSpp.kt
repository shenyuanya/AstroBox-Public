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
import android.content.Context.RECEIVER_EXPORTED
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
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
import java.util.concurrent.ConcurrentLinkedQueue
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlin.math.min

@OptIn(ObsoleteCoroutinesApi::class)
class BTSpp(private val context: Context, private val webView: WebView) {

    private val SPP_PREFIX = "00001101"          // 0x1101u → SPP
    private val MAX_PACKET_SIZE = 512            // 512 byte 分片，实测兼容性最佳

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

    private val sendScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var sendActor: SendChannel<ByteArray>? = null
    private val pendingPool = ConcurrentLinkedQueue<ByteArray>()      // 未连接或 actor 关闭时暂存

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
        } else {
            throw IllegalStateException("需要传入 Activity 作为 context，才能申请运行时权限")
        }
    }

    private suspend fun webViewLog(content: String) {
        withContext(Dispatchers.Main) {
            webView.evaluateJavascript("console.log('$content')", null)
        }
    }

    /** ------------ 扫描 ------------ **/
    @SuppressLint("MissingPermission")
    @RequiresPermission(allOf = [Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_ADMIN])
    fun startScan() {
        scannedDevices.clear()
        adapter?.let { bt ->
            if (bt.isDiscovering) bt.cancelDiscovery()

            val filter = IntentFilter(BluetoothDevice.ACTION_FOUND)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                context.registerReceiver(scanReceiver, filter, RECEIVER_EXPORTED)
            } else {
                context.registerReceiver(scanReceiver, filter)
            }
            bt.startDiscovery()
        }
    }

    @SuppressLint("MissingPermission")
    fun stopScan() {
        adapter?.cancelDiscovery()
        try { context.unregisterReceiver(scanReceiver) } catch (_: IllegalArgumentException) {}
    }

    /** ------------ 连接 ------------ **/
    @RequiresPermission(Manifest.permission.BLUETOOTH_CONNECT)
    suspend fun connect(
        context: Context,
        address: String
    ): Pair<Boolean, String?> = withContext(Dispatchers.IO) {
        var errMsg: String?
        try {
            /* 1️⃣ 运行时权限检查 */
            if (ContextCompat.checkSelfPermission(
                    context, Manifest.permission.BLUETOOTH_CONNECT
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                ActivityCompat.requestPermissions(
                    context as Activity,
                    arrayOf(Manifest.permission.BLUETOOTH_CONNECT),
                    0
                )
                errMsg = "No BLUETOOTH_CONNECT permission"
                return@withContext false to errMsg
            }

            val adapter = (context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter
                ?: return@withContext false to "BluetoothAdapter == null"
            val dev: BluetoothDevice = try {
                adapter.getRemoteDevice(address)
            } catch (iae: IllegalArgumentException) {
                errMsg = "Invalid MAC address: ${iae.message}"
                return@withContext false to errMsg
            }

            if (adapter.isDiscovering) adapter.cancelDiscovery()

            if (dev.bondState != BluetoothDevice.BOND_BONDED) {
                webViewLog("Kotlin: start bonding…")
                try {
                    dev.awaitBonded(context)
                    webViewLog("Kotlin: Bond successful!")
                } catch (e: Exception) {
                    errMsg = "Bond failed: ${e.message}"
                    return@withContext false to errMsg
                }
            }

            /* 2️⃣ socket 创建，多策略尝试 */
            val sock = tryChannel(dev, 5, 3_000)
                ?: tryChannel(dev, 1, 2_000)
                ?: trySdpUuid(dev)
                ?: return@withContext false to "No SPP channel/UUID available"

            /* 3️⃣ 完成初始化 */
            socket = sock
            inStream = sock.inputStream
            outStream = sock.outputStream
            connectedDevice = dev

            sendActor = sendScope.actor(capacity = Channel.UNLIMITED) {
                for (payload in channel) {
                    try {
                        var offset = 0
                        while (offset < payload.size) {
                            val len = min(MAX_PACKET_SIZE, payload.size - offset)
                            outStream?.write(payload, offset, len)
                            offset += len
                        }
                        outStream?.flush()
                    } catch (e: IOException) {
                        uiHandler.post { dataListener?.onError(e) }
                        break
                    }
                }
            }

            /* 3.1️⃣ 冲刷挂起的数据 */
            while (true) {
                val pending = pendingPool.poll() ?: break
                sendActor?.trySend(pending)
            }

            /* 4️⃣ 回调 */
            onConnectedCallback?.let { cb ->
                uiHandler.post { cb() }
                onConnectedCallback = null
            }
            true to null
        } catch (e: Exception) {
            errMsg = e.message
            webViewLog("Connect failed: $errMsg")
            false to errMsg
        }
    }

    fun onConnected(cb: () -> Unit) {
        if (connectedDevice != null) {
            uiHandler.post { cb() }
        } else {
            onConnectedCallback = cb
        }
    }

    /** 等待配对完成 **/
    @RequiresPermission(Manifest.permission.BLUETOOTH_CONNECT)
    suspend fun BluetoothDevice.awaitBonded(
        context: Context,
        timeoutMs: Long = 15_000L
    ) {
        if (bondState == BluetoothDevice.BOND_BONDED) return

        if (!createBond()) throw IOException("createBond() failed")

        withTimeout(timeoutMs) {
            suspendCancellableCoroutine<Unit> { cont ->
                val filter = IntentFilter(BluetoothDevice.ACTION_BOND_STATE_CHANGED)
                val receiver = object : BroadcastReceiver() {
                    @SuppressLint("MissingPermission")
                    override fun onReceive(ctx: Context?, intent: Intent?) {
                        val dev = intent?.getParcelableExtra<BluetoothDevice>(
                            BluetoothDevice.EXTRA_DEVICE
                        )
                        if (dev == null) {
                            throw NullPointerException("Device is null!!! 操你妈的怎么会出这种奇怪的问题")
                        }
                        if (dev.address != address) return
                        when (dev.bondState) {
                            BluetoothDevice.BOND_BONDED -> {
                                ctx?.unregisterReceiver(this)
                                if (cont.isActive) cont.resume(Unit)
                            }
                            BluetoothDevice.BOND_NONE -> {
                                ctx?.unregisterReceiver(this)
                                if (cont.isActive) cont.resumeWithException(IOException("Bonding failed"))
                            }
                        }
                    }
                }
                context.registerReceiver(receiver, filter)
                cont.invokeOnCancellation { context.unregisterReceiver(receiver) }
            }
        }
    }

    /** 通过 SDP UUID 尝试（secure + insecure） **/
    @SuppressLint("MissingPermission")
    private suspend fun trySdpUuid(dev: BluetoothDevice): BluetoothSocket? {
        if (!dev.fetchUuidsWithSdp()) return null

        repeat(20) {
            dev.uuids
                ?.firstOrNull { it.uuid.toString().startsWith(SPP_PREFIX, ignoreCase = true) }
                ?.let { parcel ->
                    /* insecure 优先，部分国产 ROM 只允许 insecure 连接 */
                    runCatching {
                        webViewLog("Kotlin: trySdpUuid (insecure, uuid=${parcel.uuid})")
                        val sock = dev.createInsecureRfcommSocketToServiceRecord(parcel.uuid)
                        withTimeout(6_000) { sock.connect() }
                        return sock
                    }.onFailure {
                        webViewLog("Kotlin: insecure failed, fallback secure")
                    }
                    runCatching {
                        val sock = dev.createRfcommSocketToServiceRecord(parcel.uuid) // secure
                        withTimeout(6_000) { sock.connect() }
                        return sock
                    }
                }
            delay(100)
        }
        return null
    }

    /** 通过 channel 号反射尝试（secure + insecure） **/
    private suspend fun tryChannel(
        dev: BluetoothDevice,
        ch: Int,
        timeoutMs: Long
    ): BluetoothSocket? {
        return runCatching {
            webViewLog("Kotlin: tryChannel(ch=$ch)")
            val method = runCatching {
                dev.javaClass.getMethod("createInsecureRfcommSocket", Int::class.javaPrimitiveType)
            }.getOrNull() ?: dev.javaClass.getMethod("createRfcommSocket", Int::class.javaPrimitiveType)

            val sock = method.invoke(dev, ch) as BluetoothSocket
            withTimeout(timeoutMs) { sock.connect() }
            sock
        }.getOrNull()
    }

    /** ------------ 发送 ------------ **/
    @OptIn(ExperimentalCoroutinesApi::class)
    fun send(data: ByteArray): Boolean {
        val actor = sendActor
        return if (actor != null && !actor.isClosedForSend) {
            actor.trySend(data).isSuccess
        } else {
            /* 未连接：先缓存，待连接后一次性冲刷 */
            pendingPool.add(data)
            true
        }
    }

    /** ------------ 订阅/读取 ------------ **/
    fun startSubscription() {
        if (inStream == null || readThread != null) return
        readThread = Thread {
            val buf = ByteArray(1024)
            try {
                while (!Thread.currentThread().isInterrupted) {
                    val len = inStream?.read(buf) ?: break
                    if (len <= 0) break
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

    /** ------------ 断开 ------------ **/
    @SuppressLint("MissingPermission")
    fun disconnect() {
        readThread?.interrupt(); readThread = null

        sendActor?.close()
        sendActor = null
        sendScope.coroutineContext.cancelChildren()
        pendingPool.clear()

        try { inStream?.close() } catch (_: Exception) {}
        try { outStream?.close() } catch (_: Exception) {}
        try { socket?.close() } catch (_: Exception) {}
        inStream = null; outStream = null; socket = null; connectedDevice = null
    }

    /** ------------ 扫描广播接收 ------------ **/
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
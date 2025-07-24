package com.astralsight.astrobox.plugin.btclassic_spp

import android.annotation.SuppressLint
import android.app.Activity
import android.util.Base64
import android.util.Log
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.IOException

@InvokeArg
class ConnectArg { lateinit var addr: String }

@TauriPlugin
class BtClassicSPPPlugin(private val activity: Activity): Plugin(activity) {
    private lateinit var implementation: BTSpp
    private lateinit var webView: WebView

    override fun load(webView: WebView){
        implementation = BTSpp(activity, webView)
        implementation.initPermissions()

        this.webView = webView
    }

    @SuppressLint("MissingPermission")
    @Command
    fun startScan(invoke: Invoke) {
        implementation.startScan()
        invoke.resolve()
    }

    @Command
    fun stopScan(invoke: Invoke) {
        implementation.stopScan()
        invoke.resolve()
    }

    @SuppressLint("MissingPermission")
    @Command
    fun getScannedDevices(invoke: Invoke) {
        val ret = JSArray()
        val devices = implementation.getScannedDevices()
        devices.forEach { device ->
            val newObj = JSObject()
            newObj.put("name", device.name)
            newObj.put("address", device.address)

            ret.put(newObj)
        }

        val pack = JSObject()
        pack.put("ret", ret)
        invoke.resolve(pack)
    }

    @SuppressLint("MissingPermission")
    @Command
    fun connect(invoke: Invoke) {
        val args = invoke.parseArgs(ConnectArg::class.java)
        webView.evaluateJavascript( "console.log('Kotlin: Connecting to device ${args.addr}')", null)

        CoroutineScope(Dispatchers.IO).launch {
            try {
                val isSuccessful = implementation.connect(activity, args.addr)
                val ret = JSObject()
                ret.put("ret", isSuccessful)
                invoke.resolve(ret)
            } catch (e: Exception) {
                webView.evaluateJavascript( "console.log('Kotlin: connect failed')", null)
                invoke.reject("CONNECT_ERROR", e.message, e)
            }
        }
    }

    @Command
    fun disconnect(invoke: Invoke) {
        implementation.disconnect()
        invoke.resolve()
    }

    @Command
    fun onConnected(invoke: Invoke) {
        val out = invoke.parseArgs(Channel::class.java)
        webView.evaluateJavascript("console.log('Kotlin: onConnected cb set')", null)

        implementation.onConnected {
            webView.evaluateJavascript("console.log('Kotlin: onConnected -> out.send')", null)
            out.send(JSObject())
        }

        invoke.resolve()
    }

    @SuppressLint("MissingPermission")
    @Command
    fun getConnectedDeviceInfo(invoke: Invoke) {
        val info = implementation.getConnectedDeviceInfo()
        val ret = JSObject()

        webView.evaluateJavascript("console.log('Kotlin: getConnectedDeviceInfo -> name: ${info?.name} address: ${info?.address}')", null)

        if (info != null) {
            ret.put("name", info.name)
        }
        if (info != null) {
            ret.put("address", info.address)
        }

        invoke.resolve(ret)
    }

    @Command
    fun setDataListener(invoke: Invoke) {
        val out = invoke.parseArgs(Channel::class.java)

        val callbackData = JSObject()
        implementation.setDataListener(object: BTSpp.DataListener {
            override fun onDataReceived(data: ByteArray) {
                callbackData.put("ret", Base64.encodeToString(data, Base64.NO_WRAP))
                out.send(callbackData)
            }

            override fun onError(e: IOException) {
                callbackData.put("ret", "")
                callbackData.put("err", e.toString())
                out.send(callbackData)
            }
        })

        invoke.resolve()
    }

    @Command
    fun startSubscription(invoke: Invoke) {
        implementation.startSubscription()

        invoke.resolve()
    }

    @Command
    fun send(invoke: Invoke) {
        val args = invoke.parseArgs(RustTypes.SPPSendPayload::class.java)
        val data = Base64.decode(args.b64data, Base64.DEFAULT)

        implementation.send(data)

        invoke.resolve()
    }
}

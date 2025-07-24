package com.astralsight.astrobox.plugin.btclassic_spp

import app.tauri.annotation.InvokeArg

class RustTypes {
    @InvokeArg
    class SPPDevice {
        var name: String = "";
        var address: String = "";
    }

    @InvokeArg
    class SPPSendPayload {
        var b64data: String = "";
    }
}
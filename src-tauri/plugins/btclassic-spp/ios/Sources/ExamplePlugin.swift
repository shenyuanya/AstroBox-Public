import SwiftRs
import Tauri
import UIKit
import WebKit

class PingArgs: Decodable {
  let value: String?
}

struct StringError: Error, CustomStringConvertible {
    let message: String
    var description: String { message }
}

public func throwUnsupportedSystemError() throws {
    throw StringError(message: "btclassic-spp plugin doesn't support ios")
}

class ExamplePlugin: Plugin {
    @objc public func startScan(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func stopScan(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func connect(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func startSubscription(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func send(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func getScannedDevices(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func getConnectedDeviceInfo(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func onConnected(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
    
    @objc public func setDataListener(_ invoke: Invoke) throws {
        try throwUnsupportedSystemError()
    }
}

@_cdecl("init_plugin_btclassic_spp")
func initPlugin() -> Plugin {
    return ExamplePlugin()
}

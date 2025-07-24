use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;

use windows::core::{GUID, PCWSTR};
use windows::Win32::Devices::Bluetooth::{
    BluetoothAuthenticateDeviceEx, BluetoothFindDeviceClose, BluetoothFindFirstDevice,
    BluetoothFindNextDevice, BluetoothFindFirstRadio,
    BluetoothFindRadioClose, BluetoothGetDeviceInfo, MITMProtectionNotRequired, AF_BTH,
    BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS, BLUETOOTH_FIND_RADIO_PARAMS,
    HBLUETOOTH_DEVICE_FIND, HBLUETOOTH_RADIO_FIND, SOCKADDR_BTH,
};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, FALSE, HANDLE, TRUE,
    WAIT_OBJECT_0,
};
use windows::Win32::Networking::WinSock::{
    closesocket, connect, recv, send, shutdown, socket, WSACleanup,
    WSAGetLastError, WSAStartup, INVALID_SOCKET, SEND_RECV_FLAGS, SOCKET, SOCK_STREAM,
    WSADATA, WSAEWOULDBLOCK, SD_BOTH,
};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

use crate::SPPDevice;

const BTHPROTO_RFCOMM: i32 = 3;
const RFCOMM_PORT_ANY: u32 = 0;

const SPP_SERVICE_CLASS_UUID: GUID = GUID::from_u128(0x00001101_0000_1000_8000_00805F9B34FB);

type ConnectedCallbackPtr = *const (dyn Fn() + Send + Sync + 'static);
type DataListenerPtr = *mut (dyn FnMut(Result<Vec<u8>, String>) + Send + 'static);

fn bth_addr_to_string(addr: u64) -> String {
    let bytes = addr.to_be_bytes();
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
    )
}

fn string_to_bth_addr(addr_str: &str) -> Result<u64> {
    let parts: Vec<&str> = addr_str.split(':').collect();
    if parts.len() != 6 {
        bail!("Invalid Bluetooth address format: {}", addr_str);
    }
    let mut bytes = [0u8; 8];
    for i in 0..6 {
        bytes[7 - i] = u8::from_str_radix(parts[5 - i], 16)
            .with_context(|| format!("Invalid hex byte in address: {}", parts[5 - i]))?;
    }
    Ok(u64::from_be_bytes(bytes))
}

fn string_from_utf16_char_array(chars: &[u16]) -> String {
    let len = chars.iter().take_while(|&&c| c != 0).count();
    String::from_utf16_lossy(&chars[..len])
}

fn get_first_radio_handle() -> Result<HANDLE> {
    unsafe {
        let mut params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut radio_handle = HANDLE::default();
        let find_handle: HBLUETOOTH_RADIO_FIND =
            BluetoothFindFirstRadio(&mut params, &mut radio_handle)?;
        if find_handle.is_invalid() {
            bail!("No Bluetooth radios found");
        }
        BluetoothFindRadioClose(find_handle).ok();
        if radio_handle.is_invalid() {
            bail!("Failed to acquire bluetooth radio handle");
        }
        Ok(radio_handle)
    }
}

struct ConnectedThreadHandles {
    socket: SOCKET,
    read_thread_handle: Option<thread::JoinHandle<()>>,
    stop_event: HANDLE,
}

struct GlobalState {
    wsa_initialized: bool,
    scanned_devices: Vec<SPPDevice>,
    is_scanning: bool,
    scan_stop_event: Option<HANDLE>,
    scan_thread_handle: Option<thread::JoinHandle<()>>,
    connected_device_info: Option<SPPDevice>,
    connection_handles: Option<ConnectedThreadHandles>,
    on_connected_callback: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    data_listener_callback: Option<Box<dyn FnMut(Result<Vec<u8>, String>) + Send + 'static>>,
}

impl GlobalState {
    fn new() -> Self {
        GlobalState {
            wsa_initialized: false,
            scanned_devices: Vec::new(),
            is_scanning: false,
            scan_stop_event: None,
            scan_thread_handle: None,
            connected_device_info: None,
            connection_handles: None,
            on_connected_callback: None,
            data_listener_callback: None,
        }
    }
}

static BT_STATE: Lazy<Arc<Mutex<GlobalState>>> =
    Lazy::new(|| Arc::new(Mutex::new(GlobalState::new())));

pub mod core {

    use super::*;

    fn init_winsock_if_needed() -> Result<()> {
        let mut state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for winsock init"))?;
        if !state.wsa_initialized {
            let mut wsa_data = WSADATA::default();
            let result = unsafe { WSAStartup(0x0202, &mut wsa_data) };
            if result != 0 {
                bail!("WSAStartup failed with error: {}", result);
            }
            state.wsa_initialized = true;
            info!("WinSock initialized.");
        }
        Ok(())
    }

    pub fn bluetooth_stack_cleanup() {
        let (scan_stop_event_opt, scan_thread_handle_opt, join_handle_opt, need_wsa_cleanup) = {
            let mut state = match BT_STATE.lock() {
                Ok(s) => s,
                Err(_) => {
                    warn!("Failed to lock BT_STATE for cleanup. WSA might not be cleaned up.");
                    return;
                }
            };

            let scan_stop_event_opt = if state.is_scanning {
                state.is_scanning = false;
                state.scan_stop_event.take()
            } else {
                None
            };
            let scan_thread_handle_opt = state.scan_thread_handle.take();

            let join_handle_opt = disconnect_internal(&mut state);

            let need_wsa_cleanup = state.wsa_initialized;
            if need_wsa_cleanup {
                state.wsa_initialized = false;
            }

            (
                scan_stop_event_opt,
                scan_thread_handle_opt,
                join_handle_opt,
                need_wsa_cleanup,
            )
        };

        if let Some(stop_event) = scan_stop_event_opt {
            unsafe {
                SetEvent(stop_event).ok();
            }
        }
        if let Some(handle) = scan_thread_handle_opt {
            handle
                .join()
                .unwrap_or_else(|e| warn!("Scan thread join error on cleanup: {:?}", e));
        }
        if let Some(h) = join_handle_opt {
            if thread::current().id() != h.thread().id() {
                h.join()
                    .unwrap_or_else(|e| warn!("Read thread join error on cleanup: {:?}", e));
            }
        }
        if need_wsa_cleanup {
            unsafe { WSACleanup() };
            info!("WinSock cleaned up.");
        }
    }

    pub fn start_scan_impl() -> Result<()> {
        init_winsock_if_needed()?;
        stop_scan_impl()?;
        let state_clone_for_thread = Arc::clone(&BT_STATE);
        let mut state_guard = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for start_scan"))?;

        if state_guard.is_scanning {
            info!("Scan already in progress.");
            return Ok(());
        }
        state_guard.scanned_devices.clear();

        let stop_event_handle = unsafe { CreateEventW(None, TRUE, FALSE, PCWSTR::null())? };
        state_guard.scan_stop_event = Some(stop_event_handle);
        state_guard.is_scanning = true;

        state_guard.scan_thread_handle = Some(thread::spawn(move || {
            const INQUIRY_CYCLE_PAUSE_MS: u32 = 3000;

            'outer_scan_loop: loop {
                let initial_wait_result = unsafe { WaitForSingleObject(stop_event_handle, 0) };
                if initial_wait_result == WAIT_OBJECT_0 {
                    info!("Continuous scan: Stop signal received before new inquiry cycle.");
                    break 'outer_scan_loop;
                }

                let params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
                    dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
                    fReturnAuthenticated: TRUE,
                    fReturnRemembered: TRUE,
                    fReturnUnknown: TRUE,
                    fReturnConnected: TRUE,
                    fIssueInquiry: TRUE,
                    cTimeoutMultiplier: 3,
                    hRadio: HANDLE::default(),
                };
                let mut device_info = BLUETOOTH_DEVICE_INFO {
                    dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                    ..Default::default()
                };

                info!(
                    "Continuous scan: Starting new inquiry cycle (timeout ~{}s)...",
                    params.cTimeoutMultiplier as f32 * 1.28
                );

                let find_handle_result =
                    unsafe { BluetoothFindFirstDevice(&params, &mut device_info) };

                let find_handle: HBLUETOOTH_DEVICE_FIND = match find_handle_result {
                    Ok(h) if !h.is_invalid() => h,
                    Ok(invalid_h) => {
                        error!("Continuous scan: BluetoothFindFirstDevice returned Ok with an invalid handle: {:?}. Win32 Error: {:?}. Pausing before retry.", invalid_h, unsafe { GetLastError() });
                        let pause_wait = unsafe {
                            WaitForSingleObject(stop_event_handle, INQUIRY_CYCLE_PAUSE_MS)
                        };
                        if pause_wait == WAIT_OBJECT_0 {
                            break 'outer_scan_loop;
                        }
                        continue 'outer_scan_loop;
                    }
                    Err(e) => {
                        error!("Continuous scan: BluetoothFindFirstDevice failed. Error: {:?} (Win32: {:?}). Pausing before retry.", e, unsafe { GetLastError() });
                        let pause_wait = unsafe {
                            WaitForSingleObject(stop_event_handle, INQUIRY_CYCLE_PAUSE_MS)
                        };
                        if pause_wait == WAIT_OBJECT_0 {
                            break 'outer_scan_loop;
                        }
                        continue 'outer_scan_loop;
                    }
                };

                {
                    let mut state_w = state_clone_for_thread.lock().unwrap();
                    let name_str = string_from_utf16_char_array(&device_info.szName);
                    let address_str =
                        bth_addr_to_string(unsafe { device_info.Address.Anonymous.ullLong });
                    debug!(
                        "Continuous scan found: Name: '{}', Address: {}, Paired: {}, Connected: {}",
                        name_str,
                        address_str,
                        device_info.fAuthenticated == TRUE,
                        device_info.fConnected == TRUE
                    );
                    let dev = SPPDevice {
                        name: Some(name_str),
                        address: address_str,
                    };
                    if !state_w
                        .scanned_devices
                        .iter()
                        .any(|d| d.address == dev.address)
                    {
                        state_w.scanned_devices.push(dev.clone());
                        info!("Added new device to list: {}", dev.address);
                    }
                }

                'inner_device_loop: loop {
                    let inner_wait_result = unsafe { WaitForSingleObject(stop_event_handle, 0) };
                    if inner_wait_result == WAIT_OBJECT_0 {
                        info!("Continuous scan: Stop signal received during inner device loop.");
                        unsafe {
                            BluetoothFindDeviceClose(find_handle).ok();
                        }
                        break 'outer_scan_loop;
                    }

                    device_info = BLUETOOTH_DEVICE_INFO {
                        dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                        ..Default::default()
                    };

                    match unsafe { BluetoothFindNextDevice(find_handle, &mut device_info) } {
                        Ok(_) => {
                            let mut state_w = state_clone_for_thread.lock().unwrap();
                            let name_str = string_from_utf16_char_array(&device_info.szName);
                            let address_str = bth_addr_to_string(unsafe {
                                device_info.Address.Anonymous.ullLong
                            });
                            debug!(
                                "Continuous scan found next: Name: '{}', Address: {}, Paired: {}, Connected: {}",
                                name_str, address_str, device_info.fAuthenticated == TRUE, device_info.fConnected == TRUE
                            );
                            let dev = SPPDevice {
                                name: Some(name_str),
                                address: address_str,
                            };
                            if !state_w
                                .scanned_devices
                                .iter()
                                .any(|d| d.address == dev.address)
                            {
                                state_w.scanned_devices.push(dev.clone());
                                info!("Added new device to list: {}", dev.address);
                            }
                        }
                        Err(e) => {
                            if e.code() == ERROR_NO_MORE_ITEMS.to_hresult() {
                                debug!("Continuous scan: BluetoothFindNextDevice: No more items in this cycle.");
                            } else {
                                error!("Continuous scan: BluetoothFindNextDevice error: {:?} (Win32: {:?})", e, unsafe { GetLastError() });
                            }
                            break 'inner_device_loop;
                        }
                    }
                }
                unsafe {
                    BluetoothFindDeviceClose(find_handle).ok();
                }
                info!("Continuous scan: Finished one inquiry cycle.");

                let pause_wait_result =
                    unsafe { WaitForSingleObject(stop_event_handle, INQUIRY_CYCLE_PAUSE_MS) };
                if pause_wait_result == WAIT_OBJECT_0 {
                    info!("Continuous scan: Stop signal received during pause between cycles.");
                    break 'outer_scan_loop;
                }
            }

            info!("Continuous Bluetooth device scan thread finished.");
            let mut state_w = state_clone_for_thread.lock().unwrap();
            state_w.is_scanning = false;
        }));
        Ok(())
    }

    pub fn stop_scan_impl() -> Result<()> {
        let (stop_event_opt, thread_handle_opt) = {
            let mut state = BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to lock BT_STATE for stop_scan"))?;

            if !state.is_scanning && state.scan_thread_handle.is_none() {
                return Ok(());
            }

            let stop_evt = state.scan_stop_event.take();
            let th_handle = state.scan_thread_handle.take();
            state.is_scanning = false;
            (stop_evt, th_handle)
        };

        if let Some(stop_event) = stop_event_opt {
            info!("Signaling scan thread to stop...");
            unsafe { SetEvent(stop_event).context("Failed to set scan stop event")? };
        }

        if let Some(handle) = thread_handle_opt {
            info!("Waiting for scan thread to join...");
            handle
                .join()
                .map_err(|e| anyhow!("Failed to join scan thread: {:?}", e))?;
            info!("Scan thread joined successfully.");
        }

        if let Some(stop_event) = stop_event_opt {
            if !stop_event.is_invalid() {
                info!("Closing scan stop event handle.");
                unsafe {
                    CloseHandle(stop_event).ok();
                }
            }
        }

        Ok(())
    }

    pub fn get_scanned_devices_impl() -> Result<Vec<SPPDevice>> {
        let state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for get_scanned_devices"))?;
        Ok(state.scanned_devices.clone())
    }

    fn attempt_connect(
        device_addr: u64,
        service_guid: Option<GUID>,
        port_channel: Option<u32>,
    ) -> Result<SOCKET> {
        let mut sockaddr = SOCKADDR_BTH {
            addressFamily: AF_BTH,
            btAddr: device_addr,
            serviceClassId: GUID::default(),
            port: 0,
        };

        if let Some(guid) = service_guid {
            sockaddr.serviceClassId = guid;
            sockaddr.port = RFCOMM_PORT_ANY;
        } else if let Some(ch) = port_channel {
            sockaddr.port = ch;
        } else {
            bail!("Either service_guid or port_channel must be specified for connect attempt");
        }

        let sock = unsafe { socket(AF_BTH.into(), SOCK_STREAM.into(), BTHPROTO_RFCOMM.into()) };
        if sock == INVALID_SOCKET {
            bail!("Failed to create socket: {}", unsafe {
                WSAGetLastError().0
            });
        }

        let log_service_class_id = sockaddr.serviceClassId;
        let log_port = sockaddr.port;
        info!(
            "Attempting to connect to addr {} with service_guid {:032X} and port/channel {}",
            bth_addr_to_string(device_addr),
            log_service_class_id.to_u128(),
            log_port
        );

        let connect_result = unsafe {
            connect(
                sock,
                &sockaddr as *const SOCKADDR_BTH as _,
                std::mem::size_of::<SOCKADDR_BTH>() as i32,
            )
        };
        if connect_result != 0 {
            let err_code = unsafe { WSAGetLastError().0 };
            unsafe { closesocket(sock) };
            let bail_service_class_id = sockaddr.serviceClassId;
            let bail_port = sockaddr.port;
            bail!(
                "Connection failed for service {:032X}/port {}: Win32 Error {}",
                bail_service_class_id.to_u128(),
                bail_port,
                err_code
            );
        }
        let success_service_class_id = sockaddr.serviceClassId;
        let success_port = sockaddr.port;
        info!(
            "Successfully connected using service {:032X}/port {}",
            success_service_class_id.to_u128(),
            success_port
        );
        /*
        // 不启用非阻塞模式，因为会造成一些难以处理的问题
        let mut nonblocking: u32 = 1;
        unsafe {
            ioctlsocket(sock, FIONBIO as _, &mut nonblocking);
        }*/
        Ok(sock)
    }

    pub fn connect_impl(addr_str: &str) -> Result<bool> {
        init_winsock_if_needed()?;
        stop_scan_impl()?;

        let old_join_handle_opt = {
            let mut state = BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to lock BT_STATE for connect"))?;

            if let Some(ref dev_info) = state.connected_device_info {
                if dev_info.address == addr_str {
                    info!("Already connected to this device.");
                    return Ok(true);
                }
                warn!(
                    "Connecting to a new device, disconnecting the old one: {}",
                    dev_info.address
                );
            }
            disconnect_internal(&mut state)
        };

        if let Some(h) = old_join_handle_opt {
            if thread::current().id() != h.thread().id() {
                h.join()
                    .unwrap_or_else(|e| warn!("Read thread join error: {:?}", e));
            }
        }

        let target_bth_addr = string_to_bth_addr(addr_str)?;
        let mut device_info_struct = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };
        device_info_struct.Address.Anonymous.ullLong = target_bth_addr;

        let radio_handle = match get_first_radio_handle() {
            Ok(h) => h,
            Err(e) => {
                warn!("Failed to acquire bluetooth radio handle: {:?}", e);
                HANDLE::default()
            }
        };

        let get_device_info_err =
            unsafe { BluetoothGetDeviceInfo(radio_handle, &mut device_info_struct) };
        if get_device_info_err != ERROR_SUCCESS.0 {
            warn!(
                "BluetoothGetDeviceInfo for {} failed initially or device not remembered. Error code: {}. This is expected for unbonded devices.",
                addr_str, get_device_info_err
            );
            device_info_struct.fAuthenticated = FALSE;
        } else {
            info!(
                "Device {} name: '{}', authenticated: {}",
                addr_str,
                string_from_utf16_char_array(&device_info_struct.szName),
                device_info_struct.fAuthenticated == TRUE
            );
        }

        if device_info_struct.fAuthenticated != TRUE {
            info!(
                "Device {} is not authenticated (paired). Attempting to pair now...",
                addr_str
            );
            let auth_result = unsafe {
                BluetoothAuthenticateDeviceEx(
                    None,
                    radio_handle,
                    &mut device_info_struct,
                    None,
                    MITMProtectionNotRequired,
                )
            };

            if auth_result == ERROR_SUCCESS.0 {
                info!("BluetoothAuthenticateDeviceEx call returned ERROR_SUCCESS for {}. Device should now be authenticated (paired). fAuthenticated flag is: {}", addr_str, device_info_struct.fAuthenticated == TRUE);
                if device_info_struct.fAuthenticated != TRUE {
                    warn!("Device {} pairing initiated by BluetoothAuthenticateDeviceEx, but fAuthenticated is still false. Pairing might be in progress or require further action.", addr_str);
                }
            } else {
                warn!("BluetoothAuthenticateDeviceEx call failed for {}. Error code: {}. Pairing may require user interaction or failed. Connection attempt will proceed.", addr_str, auth_result);
            }

        }
        if !radio_handle.is_invalid() {
            unsafe { CloseHandle(radio_handle).ok() };
        }

        let sock = attempt_connect(target_bth_addr, None, Some(5))
            .or_else(|e| {
                warn!(
                    "RFCOMM Channel 5 failed for {}: {:?}. Trying channel 1...",
                    addr_str, e
                );
                attempt_connect(target_bth_addr, None, Some(1))
            })
            .or_else(|e| {
                warn!(
                    "RFCOMM Channel 1 failed for {}: {:?}. Trying SPP Service UUID...",
                    addr_str, e
                );
                attempt_connect(target_bth_addr, Some(SPP_SERVICE_CLASS_UUID), None)
            })?;
        info!("SPP Connection successful to {}", addr_str);

        let cb_ptr_opt: Option<ConnectedCallbackPtr> = {
            let mut state = BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to lock BT_STATE post connection"))?;

            let stop_event = unsafe { CreateEventW(None, TRUE, FALSE, PCWSTR::null())? };
            state.connection_handles = Some(ConnectedThreadHandles {
                socket: sock,
                read_thread_handle: None,
                stop_event,
            });
            state.connected_device_info = Some(SPPDevice {
                name: Some(string_from_utf16_char_array(&device_info_struct.szName)),
                address: addr_str.to_string(),
            });

            state
                .on_connected_callback
                .as_ref()
                .map(|cb| &**cb as ConnectedCallbackPtr)
        };

        if let Some(cb_ptr) = cb_ptr_opt {
            // SAFETY: 回调存活于全局状态生命周期
            unsafe { (&*cb_ptr)() };
        }

        Ok(true)
    }

    pub fn get_connected_device_info_impl() -> Result<Option<SPPDevice>> {
        let state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for get_connected_device_info"))?;
        Ok(state.connected_device_info.clone())
    }

    pub fn on_connected_impl(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Result<()> {
        let should_call_now = {
            let state = BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to lock BT_STATE for on_connected (check)"))?;
            state.connected_device_info.is_some()
        };

        if should_call_now {
            cb();
        }

        let mut state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for on_connected (store)"))?;
        state.on_connected_callback = Some(cb);
        Ok(())
    }

    pub fn set_data_listener_impl(
        cb: Box<dyn FnMut(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<()> {
        let mut state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for set_data_listener"))?;
        state.data_listener_callback = Some(cb);
        Ok(())
    }

    pub fn start_subscription_impl() -> Result<()> {
        let (sock_copy_opt, stop_event_for_thread_opt, can_start) = {
            let state_guard = BT_STATE.lock().map_err(|_| {
                anyhow!("Failed to lock BT_STATE for start_subscription (initial check)")
            })?;

            if let Some(ref handles) = state_guard.connection_handles {
                if handles.read_thread_handle.is_none() {
                    (Some(handles.socket), Some(handles.stop_event), true)
                } else {
                    warn!("Subscription already active or read thread handle exists.");
                    (None, None, false)
                }
            } else {
                (None, None, false)
            }
        };

        if !can_start {
            if sock_copy_opt.is_none() {
                bail!("Not connected. Cannot start subscription.");
            }
            return Ok(());
        }

        let sock_copy = sock_copy_opt.unwrap();
        let stop_event_for_thread = stop_event_for_thread_opt.unwrap();

        let mut state_guard = BT_STATE.lock().map_err(|_| {
            anyhow!("Failed to lock BT_STATE for start_subscription (update handle)")
        })?;

        match state_guard.connection_handles {
            Some(ref mut handles)
                if handles.socket == sock_copy && handles.read_thread_handle.is_none() =>
            {
                let state_clone_for_thread = Arc::clone(&BT_STATE);
                handles.read_thread_handle = Some(thread::spawn(move || {
                    info!("Read thread started for socket {:?}", sock_copy);
                    let mut buffer = [0u8; 1024];
                    loop {

                        let bytes_received =
                            unsafe { recv(sock_copy, &mut buffer, SEND_RECV_FLAGS(0)) };

                        let wait_result = unsafe { WaitForSingleObject(stop_event_for_thread, 0) };

                        info!("Wait status: {:?}", wait_result);

                        if wait_result == WAIT_OBJECT_0 {
                            unsafe { CloseHandle(stop_event_for_thread).ok() };
                            info!("Read thread received stop signal.");
                            break;
                        }

                        if bytes_received > 0 {
                            let data = buffer[..bytes_received as usize].to_vec();
                            let mut maybe_cb: Option<DataListenerPtr> = None;
                            if let Ok(mut st_lock) = state_clone_for_thread.lock() {
                                maybe_cb = st_lock
                                    .data_listener_callback
                                    .as_mut()
                                    .map(|cb| &mut **cb as DataListenerPtr);
                            }
                            if let Some(cb_ptr) = maybe_cb {
                                unsafe { (&mut *cb_ptr)(Ok(data)) };
                            }
                        } else if bytes_received == 0 {
                            info!("Connection closed by peer (socket {:?}).", sock_copy);
                            let mut maybe_cb: Option<DataListenerPtr> = None;
                            if let Ok(mut st_lock) = state_clone_for_thread.lock() {
                                maybe_cb = st_lock
                                    .data_listener_callback
                                    .as_mut()
                                    .map(|cb| &mut **cb as DataListenerPtr);
                            }
                            if let Some(cb_ptr) = maybe_cb {
                                unsafe {
                                    (&mut *cb_ptr)(Err("Connection closed by peer".to_string()))
                                };
                            }
                            break; // 之后会调用 disconnect_internal 清理
                        } else {
                            // bytes_received < 0
                            let error_code = unsafe { WSAGetLastError() };
                            if error_code.0 == WSAEWOULDBLOCK.0 {
                                thread::sleep(Duration::from_millis(10));
                                continue;
                            }
                            error!(
                                "recv failed with error: {} (socket {:?})",
                                error_code.0, sock_copy
                            );
                            let mut maybe_cb: Option<DataListenerPtr> = None;
                            if let Ok(mut st_lock) = state_clone_for_thread.lock() {
                                maybe_cb = st_lock
                                    .data_listener_callback
                                    .as_mut()
                                    .map(|cb| &mut **cb as DataListenerPtr);
                            }
                            if let Some(cb_ptr) = maybe_cb {
                                unsafe {
                                    (&mut *cb_ptr)(Err(format!("Socket error: {}", error_code.0)))
                                };
                            }
                            break; // 之后会调用 disconnect_internal 清理
                        }
                    }
                    info!("Read thread stopped for socket {:?}", sock_copy);
                    // disconnect_internal 应该在主线程调用 disconnect 时或此处被动断开时被触发
                    // 如果是因为错误退出循环，上面的 break 已经使得错误通过回调传递了
                    // 此处的 disconnect_internal 主要是为了清理 BT_STATE
                    // 不需要再次触发 data_listener_callback
                    if let Ok(mut st_lock) = state_clone_for_thread.lock() {
                        let join_handle_opt = disconnect_internal(&mut st_lock);
                        drop(st_lock);
                        if let Some(h) = join_handle_opt {
                            if thread::current().id() != h.thread().id() {
                                // 避免自己join自己
                                h.join().unwrap_or_else(|e| {
                                    warn!("Read thread's internal join failed: {:?}", e)
                                });
                            }
                        }
                    }
                }));
                info!("Subscription started successfully.");
            }
            _ => {
                warn!("Could not start subscription, state might have changed or connection is different.");
            }
        }
        Ok(())
    }

    pub fn send_impl(data: &[u8]) -> Result<()> {
        let state = BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to lock BT_STATE for send"))?;
        if let Some(ref handles) = state.connection_handles {
            let mut total_sent = 0;
            while total_sent < data.len() {
                let sent = unsafe { send(handles.socket, &data[total_sent..], SEND_RECV_FLAGS(0)) };
                if sent > 0 {
                    total_sent += sent as usize;
                } else {
                    bail!("send failed with error: {}", unsafe { WSAGetLastError().0 });
                }
            }
            Ok(())
        } else {
            bail!("Not connected. Cannot send data.")
        }
    }

    fn disconnect_internal(state: &mut GlobalState) -> Option<thread::JoinHandle<()>> {
        let thread_handle_opt = if let Some(handles) = state.connection_handles.take() {
            info!("Disconnecting socket {:?}", handles.socket);
            if !handles.stop_event.is_invalid() {
                info!("SetEvent stop_event");
                unsafe {
                    SetEvent(handles.stop_event).ok();
                }
            } else {
                error!("stop_event is invalid");
            }
            let thread_handle = handles.read_thread_handle;
            unsafe {
                if shutdown(handles.socket, SD_BOTH) != 0 {
                    warn!("socket shutdown failed: {}", WSAGetLastError().0);
                }
                closesocket(handles.socket);
            }
            thread_handle
        } else {
            None
        };
        state.connected_device_info = None;
        info!("Disconnected.");
        thread_handle_opt
    }

    pub fn disconnect_impl() -> Result<()> {
        let join_handle_opt = {
            let mut state = BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to lock BT_STATE for disconnect"))?;
            disconnect_internal(&mut state)
        };

        if let Some(h) = join_handle_opt {
            if thread::current().id() != h.thread().id() {
                h.join()
                    .unwrap_or_else(|e| warn!("Read thread join error: {:?}", e));
            }
        }
        Ok(())
    }
}

pub fn cleanup_bluetooth_resources() {
    core::bluetooth_stack_cleanup();
}

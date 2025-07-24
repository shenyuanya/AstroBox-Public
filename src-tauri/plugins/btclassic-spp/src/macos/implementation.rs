use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{mpsc, Arc, Mutex};

use anyhow::{anyhow, bail, Result};
use dispatch::Queue;
use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{define_class, msg_send, ClassType, MainThreadMarker, MainThreadOnly, Message};
use objc2_foundation::{NSObject, NSString};
use objc2_io_bluetooth::{
    BluetoothRFCOMMChannelID, IOBluetoothDevice, IOBluetoothDeviceInquiry,
    IOBluetoothDeviceInquiryDelegate, IOBluetoothRFCOMMChannel, IOBluetoothRFCOMMChannelDelegate
};
use objc2_io_kit::kIOReturnSuccess;
use once_cell::sync::Lazy;

use crate::models::SPPDevice;

/* ---------- Address format helpers ---------- */
/// Convert macOS-delivered address (typically with `-` separators) to the
/// canonical `XX:XX:XX:XX:XX:XX` form used across platforms.
fn normalize_addr_from_macos(raw: &str) -> String {
    raw.replace('-', ":").to_uppercase()
}

/// Convert canonical `XX:XX:XX:XX:XX:XX` address into the format expected by
/// macOS APIs (`XX-XX-XX-XX-XX-XX`).
fn addr_to_macos_format(addr: &str) -> String {
    addr.replace(':', "-").to_uppercase()
}

/* ---------- 把闭包封送到主线程并返回结果 ---------- */
fn run_on_main_thread<F, R>(f: F) -> R
where
    F: FnOnce(MainThreadMarker) -> R + Send + 'static,
    R: Send + 'static,
{
    if let Some(mtm) = MainThreadMarker::new() {
        return f(mtm);
    }
    let (tx, rx) = mpsc::channel();
    Queue::main().exec_sync(move || {
        let mtm = MainThreadMarker::new().expect("MainThreadMarker missing on main queue");
        let _ = tx.send(f(mtm));
    });
    rx.recv().unwrap()
}

/* ---------- 全局/线程局部状态 ---------- */
struct SharedState {
    scanned_devices: Vec<SPPDevice>,
    /// true = 连续扫描循环开启；false = stop_scan_impl 已请求停止
    scan_loop_running: bool,
    connected_device_info: Option<SPPDevice>,
    on_connected_callback: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    data_listener_callback: Option<Box<dyn FnMut(Result<Vec<u8>, String>) + Send + 'static>>,
}
impl SharedState {
    fn new() -> Self {
        Self {
            scanned_devices: Vec::new(),
            scan_loop_running: false,
            connected_device_info: None,
            on_connected_callback: None,
            data_listener_callback: None,
        }
    }
}

#[derive(Default)]
struct MainThreadState {
    inquiry: Option<Retained<IOBluetoothDeviceInquiry>>,
    delegate: Option<Retained<BTDelegate>>,
    rfcomm_channel: Option<Retained<IOBluetoothRFCOMMChannel>>,
}

static SHARED_BT_STATE: Lazy<Arc<Mutex<SharedState>>> =
    Lazy::new(|| Arc::new(Mutex::new(SharedState::new())));

thread_local! {
    static MAIN_THREAD_STATE: RefCell<MainThreadState> =
        RefCell::new(MainThreadState::default());
}

/* ---------- Objective-C delegate ---------- */
define_class! {
    #[derive(Debug)]
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct BTDelegate;

    unsafe impl NSObjectProtocol for BTDelegate {}

    /* -- 设备发现回调 -- */
    unsafe impl IOBluetoothDeviceInquiryDelegate for BTDelegate {
        #[unsafe(method(deviceInquiryDeviceFound:device:))]
        fn device_inquiry_device_found_device(
            &self,
            _sender: &IOBluetoothDeviceInquiry,
            device:  &IOBluetoothDevice,
        ) {
            let raw_addr = unsafe { device.addressString() }
                .map(|s| s.to_string())
                .unwrap_or_default();
            let addr = normalize_addr_from_macos(&raw_addr);
            let name = unsafe { device.nameOrAddress() }.map(|s| s.to_string());

            let info = SPPDevice { name, address: addr.clone() };

            if let Ok(mut st) = SHARED_BT_STATE.lock() {
                if !st.scanned_devices.iter().any(|d| d.address == addr) {
                    st.scanned_devices.push(info);
                }
            }
        }

        /* -- 本轮 inquiry 结束 -- */
        #[unsafe(method(deviceInquiryComplete:error:aborted:))]
        fn device_inquiry_complete_error_aborted(
            &self,
            _sender: &IOBluetoothDeviceInquiry,
            _error:  i32,
            aborted: bool,
        ) {
            /* 1. 把 TLS 中的 inquiry 清掉，先释放 RefCell 借用 */
            MAIN_THREAD_STATE.with(|c| c.borrow_mut().inquiry = None);

            /* 2. 读取是否还要继续循环扫描 */
            let continue_loop = {
                if let Ok(st) = SHARED_BT_STATE.lock() {
                    st.scan_loop_running && !aborted
                } else {
                    false
                }
            };

            /* 3. 如果需要，再次启动新一轮 inquiry */
            if continue_loop {
                if let Err(e) = start_inquiry(self) {
                    eprintln!("Failed to restart Bluetooth inquiry: {:?}", e);
                    /* 出错就终止循环 */
                    if let Ok(mut st) = SHARED_BT_STATE.lock() {
                        st.scan_loop_running = false;
                    }
                }
            }
        }
    }

    /* -- RFCOMM 相关回调 -- */
    unsafe impl IOBluetoothRFCOMMChannelDelegate for BTDelegate {
        #[unsafe(method(rfcommChannelData:data:length:))]
        fn rfcomm_channel_data_data_length(
            &self,
            _chan: &IOBluetoothRFCOMMChannel,
            data:  *mut c_void,
            len:   usize,
        ) {
            let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
            if let Ok(mut st) = SHARED_BT_STATE.lock() {
                if let Some(cb) = st.data_listener_callback.as_mut() {
                    cb(Ok(slice.to_vec()));
                }
            }
        }

        #[unsafe(method(rfcommChannelOpenComplete:status:))]
        fn rfcomm_channel_open_complete_status(
            &self,
            chan:   &IOBluetoothRFCOMMChannel,
            status: i32,
        ) {
            if status == kIOReturnSuccess {
                MAIN_THREAD_STATE.with(|c| c.borrow_mut().rfcomm_channel = Some(chan.retain()));
                if let Ok(st) = SHARED_BT_STATE.lock() {
                    if let Some(cb) = st.on_connected_callback.as_ref() {
                        cb();
                    }
                }
            } else if let Ok(mut st) = SHARED_BT_STATE.lock() {
                st.connected_device_info = None;
                if let Some(cb) = st.data_listener_callback.as_mut() {
                    cb(Err("Connection closed".into()));
                }
            }
        }

        #[unsafe(method(rfcommChannelClosed:))]
        fn rfcomm_channel_closed(&self, chan: &IOBluetoothRFCOMMChannel) {
            unsafe {
                let _ = chan.closeChannel();
                if let Some(dev_retained) = chan.getDevice() {
                    let dev: &IOBluetoothDevice = &*dev_retained;
                    let _status: i32 = msg_send![dev, closeConnection];
                }
            }

            MAIN_THREAD_STATE.with(|c| c.borrow_mut().rfcomm_channel = None);
            if let Ok(mut st) = SHARED_BT_STATE.lock() {
                st.connected_device_info = None;
                if let Some(cb) = st.data_listener_callback.as_mut() {
                    cb(Err("Connection closed".into()));
                }
            }

            log::info!("Device disconnected. cleaning up bluetooth resources...");
            cleanup_bluetooth_resources();
        }
    }
}

impl BTDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

/* ---------- 辅助：启动一轮 inquiry ---------- */
fn start_inquiry(delegate: &BTDelegate) -> Result<()> {
    /* 新建对象 */
    let inquiry: Retained<IOBluetoothDeviceInquiry> =
        unsafe { IOBluetoothDeviceInquiry::inquiryWithDelegate(Some(delegate)) }
            .ok_or_else(|| anyhow!("Failed to create Bluetooth inquiry"))?;

    unsafe { inquiry.setUpdateNewDeviceNames(true) };
    let status = unsafe { inquiry.start() };
    if status != kIOReturnSuccess {
        bail!("Failed to start scan, error code: {}", status);
    }

    /* 保存到 TLS */
    MAIN_THREAD_STATE.with(|cell| cell.borrow_mut().inquiry = Some(inquiry));
    Ok(())
}

/* ---------- 对外跨平台接口 ---------- */
pub mod core {
    use objc2_io_bluetooth::{IOBluetoothSDPServiceRecord, IOBluetoothSDPUUID};

    use super::*;

    fn get_or_create_delegate(mtm: MainThreadMarker) -> Result<Retained<BTDelegate>> {
        MAIN_THREAD_STATE.with(|cell| {
            let mut s = cell.borrow_mut();
            if s.delegate.is_none() {
                s.delegate = Some(BTDelegate::new(mtm));
            }
            Ok(s.delegate.as_ref().unwrap().clone())
        })
    }

    /* ---- 扫描 ---- */
    pub fn start_scan_impl() -> Result<()> {
        run_on_main_thread(|mtm| {
            let delegate = get_or_create_delegate(mtm)?;

            /* 如果已经在循环扫描，则先停再启，保持语义一致 */
            stop_scan_impl().ok();

            /* 清列表并标记循环开启 */
            if let Ok(mut st) = SHARED_BT_STATE.lock() {
                st.scanned_devices.clear();
                st.scan_loop_running = true;
            }

            /* 启动第一轮 inquiry */
            start_inquiry(&delegate)?;
            Ok(())
        })
    }

    pub fn stop_scan_impl() -> Result<()> {
        run_on_main_thread(|_mtm| {
            /* 标记循环停止 */
            if let Ok(mut st) = SHARED_BT_STATE.lock() {
                st.scan_loop_running = false;
            }

            /* 取出并停止当前 inquiry（若有） */
            let current = MAIN_THREAD_STATE.with(|c| c.borrow_mut().inquiry.take());
            if let Some(inquiry) = current {
                unsafe { inquiry.stop() }; // 触发 aborted=true 回调
            }
            Ok(())
        })
    }

    pub fn get_scanned_devices_impl() -> Result<Vec<SPPDevice>> {
        Ok(SHARED_BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
            .scanned_devices
            .clone())
    }

    /* ---------- 根据 SPP UUID 解析 RFCOMM Channel ---------- */
    fn resolve_spp_channel(device: &IOBluetoothDevice) -> Option<BluetoothRFCOMMChannelID> {
        // 0x1101 = Serial Port Profile UUID-16
        const SPP_UUID16: u16 = 0x1101;

        unsafe {
            let uuid_opt = IOBluetoothSDPUUID::uuid16(SPP_UUID16);
            let uuid = uuid_opt?;

            // 同步 SDP 查询（阻塞 ≤10 s，已在主线程）
            let _ = device.performSDPQuery(None);

            let record_opt: Option<Retained<IOBluetoothSDPServiceRecord>> =
                device.getServiceRecordForUUID(Some(&*uuid));
            let record = record_opt?;

            let mut ch: BluetoothRFCOMMChannelID = 0;
            if record.getRFCOMMChannelID(&mut ch) == kIOReturnSuccess && ch != 0 {
                Some(ch)
            } else {
                None
            }
        }
    }

    /* ---- 连接 ---- */
    pub fn connect_impl(addr_str: &str) -> Result<bool> {
        let addr = addr_str.to_string();
        run_on_main_thread(move |mtm| {
            stop_scan_impl().ok();

            /* ---- 找到目标设备 ---- */
            let dev_opt: Option<Retained<IOBluetoothDevice>> = {
                let api_addr = addr_to_macos_format(&addr);
                let addr_ns = NSString::from_str(&api_addr);
                unsafe {
                    msg_send![IOBluetoothDevice::class(), deviceWithAddressString: Some(&*addr_ns)]
                }
            };
            let dev = dev_opt.ok_or_else(|| anyhow!("Device not found for {}", addr))?;

            unsafe {
                    let dev_ref: &IOBluetoothDevice = &*dev;
                    let _status: i32 = msg_send![dev_ref, closeConnection];
            }

            let delegate = get_or_create_delegate(mtm)?;

            /* 计算要尝试的 Channel 列表：SDP → 5 → 1 */
            let mut try_channels: Vec<BluetoothRFCOMMChannelID> =
                resolve_spp_channel(&dev).into_iter().collect();
            try_channels.extend([5, 1]);

            let mut last_error = None;
            for ch_id in try_channels {
                let mut chan_opt: Option<Retained<IOBluetoothRFCOMMChannel>> = None;
                let status = unsafe {
                    dev.openRFCOMMChannelAsync_withChannelID_delegate(
                        Some(&mut chan_opt),
                        ch_id,
                        Some(&*delegate),
                    )
                };
                if status == kIOReturnSuccess {
                    /* ---- 保存通道 & “正在连接” 信息 ---- */
                    if let Some(chan) = chan_opt {
                        MAIN_THREAD_STATE
                            .with(|cell| cell.borrow_mut().rfcomm_channel = Some(chan));
                    }

                    /* 提前写入 pending device，保持原 Windows 语义 */
                    SHARED_BT_STATE
                        .lock()
                        .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
                        .connected_device_info = Some(SPPDevice {
                        name: unsafe { dev.nameOrAddress() }.map(|s| s.to_string()),
                        address: addr.clone(),
                    });

                    log::info!("RFCOMM connect request sent on channel {}", ch_id);
                    return Ok(true);
                }
                last_error = Some(status);
                log::warn!("Channel {} rejected (err {})", ch_id, status);
            }

            /* 全部通道失败 */
            bail!("All RFCOMM channel attempts failed (last={:?})", last_error);
        })
    }

    pub fn get_connected_device_info_impl() -> Result<Option<SPPDevice>> {
        Ok(SHARED_BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
            .connected_device_info
            .clone())
    }

    /* ---- 回调设置 ---- */
    pub fn on_connected_impl(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Result<()> {
        SHARED_BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
            .on_connected_callback = Some(cb);
        Ok(())
    }

    pub fn set_data_listener_impl(
        cb: Box<dyn FnMut(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<()> {
        SHARED_BT_STATE
            .lock()
            .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
            .data_listener_callback = Some(cb);
        Ok(())
    }

    pub fn start_subscription_impl() -> Result<()> {
        /* macOS 的 IOBluetoothRFCOMMChannel 已自动回调数据，
        不需要额外线程，直接返回 OK */
        Ok(())
    }

    /* ---- 数据发送 & 断开 ---- */
    pub fn send_impl(data: &[u8]) -> Result<()> {
        let payload = data.to_vec();
        run_on_main_thread(move |_mtm| {
            MAIN_THREAD_STATE.with(|cell| {
                if let Some(ref chan) = cell.borrow().rfcomm_channel {
                    let ret: i32 = unsafe {
                        chan.writeSync_length(payload.as_ptr() as *mut c_void, payload.len() as u16)
                    };
                    if ret == kIOReturnSuccess {
                        Ok(())
                    } else {
                        Err(anyhow!("Failed to send data, error code: {}", ret))
                    }
                } else {
                    Err(anyhow!("Device not connected, cannot send data"))
                }
            })
        })
    }

    pub fn disconnect_impl() -> Result<()> {
        run_on_main_thread(|_mtm| {
            let maybe_chan = MAIN_THREAD_STATE.with(|c| c.borrow_mut().rfcomm_channel.take());
            if let Some(chan) = maybe_chan {
                let status = unsafe { chan.closeChannel() };
                unsafe {
                    if let Some(dev_retained) = chan.getDevice() {
                        let dev: &IOBluetoothDevice = &*dev_retained;
                        let _status: i32 = msg_send![dev, closeConnection];
                    }
                }
                if status != kIOReturnSuccess {
                    eprintln!("Failed to close RFCOMM channel, error code: {}", status);
                }
            }
            SHARED_BT_STATE
                .lock()
                .map_err(|_| anyhow!("Failed to acquire Bluetooth state lock"))?
                .connected_device_info = None;
            Ok(())
        })
    }
}

/* ---------- 全局清理 --------- */
pub fn cleanup_bluetooth_resources() {
    let _ = core::disconnect_impl();
    let _ = core::stop_scan_impl();
}

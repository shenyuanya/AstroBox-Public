// plugins/btclassic-spp/src/linux/implementation.rs

use crate::models::SPPDevice;
use anyhow::{anyhow, Result};
use bluer::rfcomm::{SocketAddr, Stream};
use bluer::{Adapter, AdapterEvent, Address, DiscoveryFilter, DiscoveryTransport, Session};
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

// 创建一个全局、持久化的 Tokio 运行时，专门用于蓝牙操作
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

struct GlobalState {
    adapter: Option<Adapter>,
    scanned_devices: Vec<SPPDevice>,
    scan_stop: Option<Arc<AtomicBool>>,
    scan_thread: Option<JoinHandle<()>>,
    socket_stream: Option<Arc<Mutex<Stream>>>,
    read_stop: Option<Arc<AtomicBool>>,
    read_thread: Option<JoinHandle<()>>,
    connected_device_info: Option<SPPDevice>,
    on_connected_callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    data_listener_callback: Option<Arc<Mutex<Box<dyn FnMut(Result<Vec<u8>, String>) + Send>>>>,
}

impl GlobalState {
    fn new() -> Self {
        tokio::task::block_in_place(|| {
            let adapter = RUNTIME.block_on(async {
                let session = match Session::new().await {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to create bluer session: {}", e);
                        return None;
                    }
                };
                let adapter = match session.default_adapter().await {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!("Failed to get default adapter: {}", e);
                        return None;
                    }
                };
                if let Err(e) = adapter
                    .set_discovery_filter(DiscoveryFilter {
                        transport: DiscoveryTransport::BrEdr,
                        ..Default::default()
                    })
                    .await
                {
                    log::error!("Failed to set discovery filter: {}", e);
                }
                Some(adapter)
            });

            Self {
                adapter,
                scanned_devices: Vec::new(),
                scan_stop: None,
                scan_thread: None,
                socket_stream: None,
                read_stop: None,
                read_thread: None,
                connected_device_info: None,
                on_connected_callback: None,
                data_listener_callback: None,
            }
        })
    }
}

static STATE: Lazy<Arc<std::sync::Mutex<GlobalState>>> =
    Lazy::new(|| Arc::new(std::sync::Mutex::new(GlobalState::new())));

pub fn init_bluetooth_stack() {
    log::info!("Initializing Linux Bluetooth stack...");
    Lazy::force(&STATE);
    log::info!("Linux Bluetooth stack initialized.");
}

pub mod core {
    use super::*;

    pub fn start_scan_impl() -> Result<()> {
        stop_scan_impl()?;
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_thread = stop_flag.clone();
        let state_clone = STATE.clone();

        let handle = RUNTIME.spawn(async move {
            let adapter = match state_clone.lock() {
                Ok(guard) => guard.adapter.clone(),
                Err(_) => {
                    log::error!("Failed to lock state for scanning");
                    return;
                }
            };

            let adapter = match adapter {
                Some(a) => a,
                None => {
                    log::error!("Bluetooth adapter not available for scanning.");
                    return;
                }
            };

            if let Err(e) = adapter.set_powered(true).await {
                log::error!("Failed to power on adapter: {}", e);
                return;
            }

            let mut scan_stream = match adapter.discover_devices().await {
                Ok(s) => Box::pin(s),
                Err(e) => {
                    log::error!("Failed to start device discovery: {}", e);
                    return;
                }
            };

            log::info!("Linux Bluetooth scan started.");

            while !stop_flag_thread.load(Ordering::SeqCst) {
                tokio::select! {
                    biased;
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                    event_opt = scan_stream.next() => {
                        if let Some(event) = event_opt {
                            match event {
                                AdapterEvent::DeviceAdded(addr) => {
                                    if let Ok(device) = adapter.device(addr) {
                                        let name_res = device.name().await;
                                        let info = SPPDevice {
                                            name: name_res.unwrap_or(None),
                                            address: addr.to_string(),
                                        };
                                        let mut st = state_clone.lock().unwrap();
                                        if !st.scanned_devices.iter().any(|d| d.address == info.address) {
                                            log::info!("Found device: {} ({})", info.address, info.name.as_deref().unwrap_or("N/A"));
                                            st.scanned_devices.push(info);
                                        }
                                    }
                                }
                                AdapterEvent::DeviceRemoved(addr) => {
                                     let mut st = state_clone.lock().unwrap();
                                     st.scanned_devices.retain(|d| d.address != addr.to_string());
                                }
                                _ => {}
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            log::info!("Linux Bluetooth scan stopped.");
            let mut st = state_clone.lock().unwrap();
            st.scan_thread = None;
            st.scan_stop = None;
        });

        let mut st = STATE.lock().unwrap();
        st.scanned_devices.clear();
        st.scan_stop = Some(stop_flag);
        st.scan_thread = Some(handle);
        Ok(())
    }

    pub fn stop_scan_impl() -> Result<()> {
        let (stop_opt, thread_opt) = {
            let mut st = STATE.lock().unwrap();
            (st.scan_stop.take(), st.scan_thread.take())
        };
        if let Some(flag) = stop_opt {
            flag.store(true, Ordering::SeqCst);
        }
        if let Some(h) = thread_opt {
            h.abort();
        }
        Ok(())
    }

    pub fn get_scanned_devices_impl() -> Result<Vec<SPPDevice>> {
        let st = STATE.lock().unwrap();
        Ok(st.scanned_devices.clone())
    }

    pub fn connect_impl(addr_str: &str) -> Result<bool> {
        stop_scan_impl()?;
        disconnect_impl()?;

        let addr_clone = addr_str.to_string();

        // 修复：使用 block_in_place，并让其闭包返回 Result
        tokio::task::block_in_place(|| {
            RUNTIME.block_on(async move {
                let addr: Address = addr_clone.parse()
                    .map_err(|e| anyhow!("Invalid address format: {}", e))?;

                let channels_to_try = [5, 1];
                let mut stream: Option<Stream> = None;

                for &channel in &channels_to_try {
                    let sock_addr = SocketAddr::new(addr, channel);
                    log::info!("Attempting to connect to {} on channel {}", addr, channel);
                    match tokio::time::timeout(Duration::from_secs(10), Stream::connect(sock_addr)).await {
                        Ok(Ok(s)) => {
                            log::info!("Successfully connected on channel {}", channel);
                            stream = Some(s);
                            break;
                        }
                        Ok(Err(e)) => log::warn!("Failed to connect on channel {}: {}", channel, e),
                        Err(_) => log::warn!("Timeout connecting on channel {}", channel),
                    }
                }
                
                if let Some(connected_stream) = stream {
                    let socket_arc = Arc::new(Mutex::new(connected_stream));
                    let cb_opt = {
                        let mut name: Option<String> = None;
                        let mut st = STATE.lock().unwrap();
                        if let Some(adapter) = st.adapter.clone() {
                            if let Ok(device) = adapter.device(addr) {
                                name = device.name().await.unwrap_or(None)
                            }
                        }
                        st.socket_stream = Some(socket_arc.clone());
                        st.connected_device_info = Some(SPPDevice {
                            name,
                            address: addr_clone,
                        });
                        st.on_connected_callback.clone()
                    };
                    if let Some(cb) = cb_opt {
                        cb();
                    }
                    Ok(true)
                } else {
                    Err(anyhow!("Failed to connect on all attempted channels"))
                }
            })
        })
    }

    pub fn get_connected_device_info_impl() -> Result<Option<SPPDevice>> {
        let st = STATE.lock().unwrap();
        Ok(st.connected_device_info.clone())
    }

    pub fn on_connected_impl(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Result<()> {
        let should_call = {
            let st = STATE.lock().unwrap();
            st.connected_device_info.is_some()
        };
        if should_call {
            cb();
        }
        let mut st = STATE.lock().unwrap();
        st.on_connected_callback = Some(Arc::new(cb));
        Ok(())
    }

    pub fn set_data_listener_impl(
        cb: Box<dyn FnMut(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<()> {
        let mut st = STATE.lock().unwrap();
        st.data_listener_callback = Some(Arc::new(Mutex::new(cb)));
        Ok(())
    }

    pub fn start_subscription_impl() -> Result<()> {
        let (socket_arc, cb_arc, stop_flag) = {
            let mut st = STATE.lock().unwrap();
            if st.read_thread.is_some() { return Ok(()); }
            let sock = st.socket_stream.clone().ok_or_else(|| anyhow!("Not connected"))?;
            let cb = st.data_listener_callback.clone().ok_or_else(|| anyhow!("Data listener not set"))?;
            let flag = Arc::new(AtomicBool::new(false));
            st.read_stop = Some(flag.clone());
            (sock, cb, flag)
        };

        let handle = RUNTIME.spawn(async move {
            let mut buf = [0u8; 1024];
            while !stop_flag.load(Ordering::SeqCst) {
                let mut sock_lock = socket_arc.lock().await;
                tokio::select! {
                    biased;
                    _ = tokio::time::sleep(Duration::from_millis(10)) => continue,
                    read_res = sock_lock.read(&mut buf) => {
                        match read_res {
                            Ok(0) => {
                                log::info!("Connection closed by peer.");
                                let mut f = cb_arc.lock().await;
                                f(Err("Connection closed".into()));
                                break;
                            }
                            Ok(n) => {
                                let data = buf[..n].to_vec();
                                let mut f = cb_arc.lock().await;
                                f(Ok(data));
                            }
                            Err(e) => {
                                log::error!("Socket read error: {}", e);
                                let mut f = cb_arc.lock().await;
                                f(Err(e.to_string()));
                                break;
                            }
                        }
                    }
                }
            }
        });

        let mut st = STATE.lock().unwrap();
        st.read_thread = Some(handle);
        Ok(())
    }

    pub fn send_impl(data: &[u8]) -> Result<()> {
        let socket_arc = {
            let st = STATE.lock().unwrap();
            st.socket_stream.clone().ok_or_else(|| anyhow!("Not connected"))?
        };
        let data_clone = data.to_vec();

        // 修复：使用 block_in_place，并让其闭包返回 Result
        tokio::task::block_in_place(|| {
            RUNTIME.block_on(async {
                let mut sock = socket_arc.lock().await;
                sock.write_all(&data_clone).await?;
                sock.flush().await?;
                Ok(())
            })
        })
    }

    pub fn disconnect_impl() -> Result<()> {
        let (socket_opt, stop_opt, thread_opt) = {
            let mut st = STATE.lock().unwrap();
            let s = st.socket_stream.take();
            let stop = st.read_stop.take();
            let th = st.read_thread.take();
            st.connected_device_info = None;
            (s, stop, th)
        };
        if let Some(flag) = stop_opt {
            flag.store(true, Ordering::SeqCst);
        }
        if let Some(h) = thread_opt {
            h.abort();
        }
        drop(socket_opt);
        Ok(())
    }
}
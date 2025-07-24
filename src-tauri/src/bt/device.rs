use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use btclassic_spp::SPPDevice;
use serde::{Deserialize, Serialize};
use tauri_plugin_blec::models::{BleDevice, ScanFilter, WriteType};
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BTDevice {
    pub handle: Connection,
    pub ble_services: Option<Vec<tauri_plugin_blec::models::Service>>, // ble服务列表 仅handle为BleDevice时可用
}

impl BTDevice {
    pub fn scan<F>(timeout: i32, scan_type: ConnectType, mut cb: F) -> Result<()>
    where
        F: FnMut(Vec<BTDevice>) + Send + 'static,
    {
        let timeout = timeout as u64;

        match scan_type {
            ConnectType::BLE => {
                let handler = super::get_ble_handler();
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);

                thread::spawn(move || {
                    tauri::async_runtime::block_on(async {
                        handler
                            .discover(Some(tx), timeout, ScanFilter::None)
                            .await
                            .expect("Failed to start BLE discover");
                    });

                    let start = Instant::now();
                    while start.elapsed() < Duration::from_millis(timeout) {
                        if let Some(batch) = tauri::async_runtime::block_on(rx.recv()) {
                            let devices = batch
                                .into_iter()
                                .map(|d| BTDevice {
                                    handle: Connection::BLE(d),
                                    ble_services: None,
                                })
                                .collect();
                            cb(devices);
                        } else {
                            break;
                        }
                    }
                });
                Ok(())
            }

            ConnectType::SPP => {
                thread::spawn(move || {
                    if let Err(e) = super::get_spp_handle().start_scan() {
                        log::error!("SPP start_scan failed: {}", e);
                        return;
                    }
                    let start = Instant::now();

                    while start.elapsed() < Duration::from_millis(timeout) {
                        match super::get_spp_handle().get_scanned_devices() {
                            Ok(result) => {
                                let devices = result
                                    .ret
                                    .into_iter()
                                    .map(|d| BTDevice {
                                        handle: Connection::SPP(d),
                                        ble_services: None,
                                    })
                                    .collect();
                                cb(devices);
                            }
                            Err(e) => {
                                log::warn!("Failed to get SPP devices: {}", e);
                            }
                        }
                        thread::sleep(Duration::from_secs(1));
                    }

                    if let Err(e) = super::get_spp_handle().stop_scan() {
                        log::error!("SPP stop_scan failed: {}", e);
                    }
                });
                Ok(())
            }
        }
    }

    pub async fn connect(
        addr: &String,
        connect_type: ConnectType,
    ) -> Result<BTDevice, anyhow::Error> {
        match connect_type {
            ConnectType::BLE => {
                super::get_ble_handler()
                    .connect(&addr, tauri_plugin_blec::OnDisconnectHandler::None)
                    .await
                    .context("Failed to connect")?;

                let device = super::get_ble_handler()
                    .connected_device()
                    .await
                    .context("No connected device")?;

                let services = super::get_ble_handler()
                    .discover_services(&device.address)
                    .await
                    .context("Failed to discover BLE services")?;

                for service in &services {
                    println!("Found service {}", service.uuid);
                    for chara in &service.characteristics {
                        println!("  Chara UUID={}", chara.uuid);
                    }
                }

                Ok(BTDevice {
                    handle: Connection::BLE(device),
                    ble_services: Some(services),
                })
            }
            ConnectType::SPP => {
                let (tx, rx) = oneshot::channel();
                let tx_holder = Arc::new(Mutex::new(Some(tx)));

                super::get_spp_handle().on_connected({
                    let tx_holder = Arc::clone(&tx_holder);
                    move || {
                        log::info!("SPP Connected!");
                        let tx_holder2 = Arc::clone(&tx_holder);
                        tauri::async_runtime::spawn(async move {
                            let info = super::get_spp_handle().get_connected_device_info();
                            if let Some(tx) = tx_holder2.lock().unwrap().take() {
                                let _ = tx.send(info);
                            }
                        });
                    }
                })?;

                log::info!("BTDevice connecting...");
                let conn_result = super::get_spp_handle()
                    .connect(addr)
                    .context("Failed to initiate SPP connect")?;
                if !conn_result.ret {
                    bail!("SPP Connect failed!");
                }

                let device_info = rx
                    .await
                    .context("SPP connection failed or timed out")?
                    .context("SPP device info missing")?;

                log::info!("Connected to SPP Device: addr={}", &addr);

                Ok(BTDevice {
                    handle: Connection::SPP(device_info),
                    ble_services: None,
                })
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), anyhow::Error> {
        match &self.handle {
            Connection::BLE(_ble_device) => {
                super::get_ble_handler().disconnect().await?;
            }
            Connection::SPP(_sppdevice) => {
                super::get_spp_handle()
                    .disconnect()
                    .map_err(|e| anyhow::anyhow!("SPP disconnect failed: {}", e))?;
            }
        }

        Ok(())
    }

    pub async fn subscribe(
        &self,
        cb: Arc<dyn Fn(Result<Vec<u8>, String>) + Send + Sync>,
        characteristic: Option<Uuid>,
    ) -> Result<(), anyhow::Error> {
        match &self.handle {
            Connection::BLE(_ble_device) => {
                let cb_clone = cb.clone();
                super::get_ble_handler()
                    .subscribe(
                        characteristic
                            .ok_or_else(|| anyhow::anyhow!("missing char for BLE subscribe"))?,
                        move |data| {
                            cb_clone(Ok(data.to_vec()));
                        },
                    )
                    .await
                    .context("BLE subscribe call failed")?;
            }
            Connection::SPP(spp_device_info) => {
                let cb_clone = cb.clone();
                let active_spp_address = spp_device_info.address.clone();

                super::get_spp_handle()
                    .set_data_listener(move |result_data| {
                        let device_addr_for_clear_check = active_spp_address.clone();
                        match &result_data {
                            Ok(data) => cb_clone(Ok(data.clone())),
                            Err(e) => {
                                cb_clone(Err(e.clone()));

                                let e_cloned = e.clone();
                                let addr_cloned = device_addr_for_clear_check.clone();

                                tauri::async_runtime::spawn(async move {
                                    let mut should_clear = false;
                                    let global_dev_guard =
                                        crate::miwear::CONNECTED_DEVICE.read().await;
                                    if let Some(global_dev_arc) =
                                        global_dev_guard.as_ref().cloned()
                                    {
                                        let global_dev_state =
                                            global_dev_arc.state.read().await;
                                        if global_dev_state.addr == addr_cloned {
                                            should_clear = true;
                                        }
                                    }
                                    drop(global_dev_guard);
                                    if should_clear {
                                        crate::miwear::set_connected_device(None).await;
                                        log::error!(
                                            "SPP connection error for device {}, cleared CONNECTED_DEVICE. Error: {}",
                                            addr_cloned, e_cloned
                                        );
                                    }
                                });
                            }
                        }
                    })
                    .map_err(|e| anyhow::anyhow!("SPP set_data_listener failed: {}", e))?;
                super::get_spp_handle()
                    .start_subscription()
                    .map_err(|e| anyhow::anyhow!("SPP start_subscription failed: {}", e))?;
            }
        }
        Ok(())
    }

    pub async fn send(
        &self,
        data: Vec<u8>,
        characteristic: Option<Uuid>,
    ) -> anyhow::Result<(), anyhow::Error> {
        match &self.handle {
            Connection::BLE(_ble_device) => {
                super::get_ble_handler()
                    .send_data(
                        characteristic
                            .ok_or_else(|| anyhow::anyhow!("missing char"))?,
                        &data,
                        WriteType::WithoutResponse,
                    )
                    .await
                    .context("BLE send_data failed")?;
                Ok(())
            }
            Connection::SPP(_sppdevice) => {
                super::get_spp_handle()
                    .send(&data)
                    .map_err(|e| anyhow::anyhow!("SPP send failed: {}", e))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Connection {
    BLE(BleDevice),
    SPP(SPPDevice),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectType {
    BLE,
    SPP,
}

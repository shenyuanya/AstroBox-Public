use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use uuid::Uuid;

use crate::{
    bt::device::{BTDevice, ConnectType},
    community::provider::official::OfficialProvider,
    miwear::{device::thirdpartyapp::on_thirdparty_app, network_stack},
    pb::{self},
    tools::{to_hex_string, uuid_contains},
};

use super::{
    bleuuids::{BLE_UUID_KEYWORD_XIAOMI_RECV, BLE_UUID_KEYWORD_XIAOMI_SENT},
    packet::{self, Channel, OpCode, PacketData},
};

pub mod auth;
pub mod firmware;
pub mod hello;
pub mod mass;
pub mod models;
pub mod resutils;
pub mod system;
pub mod thirdpartyapp;
pub mod watchface;
pub mod error;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MiWearState {
    pub name: String,
    pub addr: String,
    pub authkey: String,
    pub bleservice: MiWearBleCharaUuid,
    pub max_frame_size: usize,
    pub sec_keys: Option<SecurityKeys>,
    pub network_mtu: u16,
    pub codename: String,
}

/// 公共常量：MiWear 请求默认超时时间
pub(super) const REQ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(5_000);

type ProtoKey = (u32, u32);

/// 核心设备结构
pub struct MiWearDevice {
    /// 设备信息（RwLock ⇒ 读并发，写独占且短暂）
    pub state: RwLock<MiWearState>,
    /// 待回应表：seq → oneshot sender
    pub pending_seq: DashMap<u8, oneshot::Sender<PacketData>>,
    /// 待 ACK 表：btrecv -> oneshot sender
    pub pending_ack: DashMap<(), oneshot::Sender<()>>,
    /// 待回应表：protokey → oneshot sender
    pub pending_proto: DashMap<ProtoKey, oneshot::Sender<pb::protocol::WearPacket>>,
    /// 订阅回调表：type → Vec<callback>
    pub proto_subscribers: DashMap<u32, Vec<Arc<dyn Fn(pb::protocol::WearPacket) + Send + Sync>>>,
    /// 序列号计数器
    pub seq: AtomicU8,
    /// 蓝牙抽象（BLE / SPP）
    pub btdevice: BTDevice,
    /// 用于向网络栈发送数据的 Sender
    pub network_tx: Mutex<Option<mpsc::Sender<Vec<u8>>>>,
    /// 已知第三方应用信息表：包名 → AppInfo
    pub app_info_table: DashMap<String, crate::miwear::device::thirdpartyapp::AppInfo>,
    /// 上行数据包临时存储buffer
    pub recv_buffer: Mutex<Vec<u8>>,
    /// 发送锁，用于在 Mass 传输时阻塞其他数据包
    pub send_lock: Mutex<()>,
    /// 是否处于mass传输状态,
    pub is_sending_mass: Mutex<bool>,
    /// 命令池，用于按优先级顺序发送数据包
    pub cmd_pool: Arc<crate::miwear::command_pool::CommandPool>,
    /// 下行速度
    pub network_write_speed: Mutex<f64>,
    /// 上行速度
    pub network_read_speed: Mutex<f64>, 
}

impl MiWearDevice {
    /* ───────────── 连接 ───────────── */
    pub async fn connect(addr: String, name: String) -> anyhow::Result<Arc<Self>> {
        let connect_type = crate::config::read(|c| c.clone().connect_type);

        /* 1. 建立物理连接 */
        let device = match BTDevice::connect(&addr, connect_type.clone())
            .await
            .map_err(|e| e.to_string())
        {
            Ok(device) => device,
            Err(e) => return Err(anyhow!("连接失败: {}", e)),
        };

        /* 2. 提取名字 / 地址 */
        let (mut device_name, device_address) = match &device.handle {
            crate::bt::device::Connection::BLE(ble) => (ble.name.clone(), ble.address.clone()),
            crate::bt::device::Connection::SPP(spp) => {
                (spp.name.clone().unwrap_or_default(), spp.address.clone())
            }
        };

        if device_name.is_empty() || device_name == "" {
            device_name = name.clone();
        }

        /* 3. 枚举 BLE 服务（Android SPP 无需） */
        let mut charas = MiWearBleCharaUuid {
            recv: Uuid::nil(),
            sent: Uuid::nil(),
        };
        if let Some(services) = device.ble_services.as_ref() {
            for service in services {
                for chara in &service.characteristics {
                    if uuid_contains(&service.uuid, "fe95", true)
                        && uuid_contains(&chara.uuid, BLE_UUID_KEYWORD_XIAOMI_RECV, false)
                    {
                        charas.recv = chara.uuid;
                    }
                    if uuid_contains(&service.uuid, "fe95", true)
                        && uuid_contains(&chara.uuid, BLE_UUID_KEYWORD_XIAOMI_SENT, false)
                    {
                        charas.sent = chara.uuid;
                    }
                }
            }
        }

        let recv_uuid = charas.recv;

        let mut max_frame_size = 244;
        if connect_type == ConnectType::SPP {
            max_frame_size = 977; // 1004(max size) - 27(bluetooth packet header)
        }

        /* 4. 构造核心对象 */
        let core = Arc::new_cyclic(|weak| Self {
            state: RwLock::new(MiWearState {
                name: device_name,
                addr: device_address,
                authkey: String::new(),
                bleservice: charas,
                max_frame_size,
                sec_keys: None,
                network_mtu: 800, /* 默认800 此值过大会导致表端buffer溢出 极限大概在900左右 设置为900会导致不稳定 */
                codename: String::new(),
            }),
            pending_seq: DashMap::new(),
            pending_ack: DashMap::new(),
            pending_proto: DashMap::new(),
            proto_subscribers: DashMap::new(),
            seq: AtomicU8::new(0),
            btdevice: device.clone(),
            network_tx: Mutex::new(None),
            app_info_table: DashMap::new(),
            recv_buffer: Mutex::new(Vec::new()),
            send_lock: Mutex::new(()),
            is_sending_mass: Mutex::new(false),
            cmd_pool: crate::miwear::command_pool::CommandPool::new(weak.clone()),
            network_write_speed: Mutex::new(0.0),
            network_read_speed: Mutex::new(0.0),
        });

        let network_sender = network_stack::start_network_stack(core.clone());
        *core.network_tx.lock().await = Some(network_sender);

        log::info!("Network stack is running now.");

        /* 5. 启动订阅 → 将数据投递到 handle_bt_packet */
        let dev_clone = Arc::clone(&core);
        device
            .subscribe(
                Arc::new(move |data_result| {
                    let dev2 = Arc::clone(&dev_clone);
                    tauri::async_runtime::spawn(async move {
                        match data_result {
                            Ok(data) => {
                                if data.starts_with(&[0xBA, 0xDC]) {
                                    super::btrecv::handle_bt_packet(dev2, data).await;
                                } else {
                                    dev2.push_recv_data(data).await;

                                    match dev2.pop_bt_packet().await {
                                        Some(value) => {
                                            super::btrecv::handle_bt_packet(dev2, value).await;
                                        }
                                        None => {}
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("BT subscription error: {}", e);
                                crate::miwear::set_connected_device(None).await;
                                log::info!(
                                    "Device disconnected due to error, cleaned CONNECTED_DEVICE"
                                );
                                let _ = dev2.disconnect().await;
                            }
                        }
                    });
                }),
                Some(recv_uuid),
            )
            .await?;

        log::info!("Listening messages from device...");

        let state_clone = core.state.read().await.clone();

        let mtu = crate::config::read(|c| {
            c.paired_devices
                .iter()
                .find(|dev| dev.addr == state_clone.addr)
                .map_or(0, |dev| dev.network_mtu)
        });

        core.state.write().await.network_mtu = mtu;

        let need_push = !crate::config::read(|c| {
            c.paired_devices
                .iter()
                .any(|dev| dev.addr == state_clone.addr)
        });

        crate::config::write(|c| {
            c.current_device = Some(state_clone.clone());
            if need_push {
                c.paired_devices.push(state_clone);
            }
        });

        let dev_clone_proto = Arc::clone(&core);
        core.subscribe_proto(
            pb::protocol::wear_packet::Type::ThirdpartyApp as u32,
            Arc::new(move |packet| {
                on_thirdparty_app(dev_clone_proto.clone(), packet);
            }),
        );

        log::info!("[MiWearDevice] Connected to device successfully!");

        Ok(core)
    }

    /* ───────────── 断开连接 ───────────── */
    pub async fn disconnect(&self) -> anyhow::Result<()> {
        *self.network_tx.lock().await = None;

        let device_addr = self.state.read().await.addr.clone();

        self.btdevice.disconnect().await?;
        log::info!(
            "MiWearDevice::disconnect: Successfully disconnected from Bluetooth device {}",
            &device_addr
        );

        crate::miwear::set_connected_device(None).await;
        log::info!(
            "MiWearDevice::disconnect: Cleared CONNECTED_DEVICE for {}",
            &device_addr
        );
        crate::miwear::notify_disconnect();

        Ok(())
    }

    /* ───────────── 发送 ───────────── */
    pub async fn send(&self, payload: Vec<u8>) -> anyhow::Result<()> {
        let chara = { self.state.read().await.bleservice.sent };
        match &self.btdevice.handle {
            crate::bt::device::Connection::BLE(_) => self.btdevice.send(payload, Some(chara)).await,
            crate::bt::device::Connection::SPP(_) => self.btdevice.send(payload, None).await,
        }
    }

    pub(super) async fn send_fragments(&self, frame: Vec<u8>) -> anyhow::Result<()> {
        let state = self.state.read().await;

        if frame.len() <= state.max_frame_size {
            self.send(frame).await
        } else {
            for chunk in frame.chunks(state.max_frame_size) {
                tokio::time::sleep(Duration::from_millis(
                    crate::config::read(|c| c.clone().fragments_send_delay).into(),
                ))
                .await;
                self.send(chunk.to_vec()).await?;
            }
            Ok(())
        }
    }

    pub async fn send_miwear_pkt(
        self: &Arc<Self>,
        channel: packet::Channel,
        op: packet::OpCode,
        payload: &[u8],
    ) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd_pool
            .push(crate::miwear::command_pool::Command {
                channel,
                op,
                payload: payload.to_vec(),
                kind: crate::miwear::command_pool::CommandKind::Send,
                timeout: None,
                responder: tx,
            })
            .await;
        rx.await??;
        Ok(())
    }

    pub async fn send_miwear_pkt_wait_ack(
        self: &Arc<Self>,
        channel: packet::Channel,
        op: packet::OpCode,
        payload: &[u8],
        timeout: Option<Duration>,
    ) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd_pool
            .push(crate::miwear::command_pool::Command {
                channel,
                op,
                payload: payload.to_vec(),
                kind: crate::miwear::command_pool::CommandKind::WaitAck,
                timeout,
                responder: tx,
            })
            .await;
        rx.await??;
        Ok(())
    }

    pub async fn send_miwear_pkt_register_ack(
        self: &Arc<Self>,
        channel: packet::Channel,
        op: packet::OpCode,
        payload: &[u8],
    ) -> anyhow::Result<oneshot::Receiver<()>> {
        let (tx, rx_outer) = oneshot::channel();
        self.cmd_pool
            .push(crate::miwear::command_pool::Command {
                channel,
                op,
                payload: payload.to_vec(),
                kind: crate::miwear::command_pool::CommandKind::RegisterAck { unlocked: false },
                timeout: None,
                responder: tx,
            })
            .await;
        match rx_outer.await?? {
            crate::miwear::command_pool::CommandResponse::AckReceiver(rx) => Ok(rx),
            _ => Err(anyhow!("unexpected response")),
        }
    }

    pub async fn send_miwear_pkt_register_ack_unlocked(
        self: &Arc<Self>,
        channel: packet::Channel,
        op: packet::OpCode,
        payload: &[u8],
    ) -> anyhow::Result<oneshot::Receiver<()>> {
        let (tx, rx_outer) = oneshot::channel();
        self.cmd_pool
            .push(crate::miwear::command_pool::Command {
                channel,
                op,
                payload: payload.to_vec(),
                kind: crate::miwear::command_pool::CommandKind::RegisterAck { unlocked: true },
                timeout: None,
                responder: tx,
            })
            .await;
        match rx_outer.await?? {
            crate::miwear::command_pool::CommandResponse::AckReceiver(rx) => Ok(rx),
            _ => Err(anyhow!("unexpected response")),
        }
    }

    pub async fn request(
        &self,
        channel: Channel,
        op: OpCode,
        payload: &[u8],
        timeout: Option<Duration>,
    ) -> anyhow::Result<PacketData> {
        let (seq, frame) = self.build_frame(channel, op, payload).await?;
        let (tx, rx) = oneshot::channel();
        self.pending_seq.insert(seq, tx);
        let _guard = self.send_lock.lock().await;
        self.send_fragments(frame).await?;
        Ok(tokio::time::timeout(timeout.unwrap_or(REQ_TIMEOUT), rx).await??)
    }

    pub async fn request_proto(
        &self,
        channel: Channel,
        op: OpCode,
        payload: &[u8],
        expect_type: u32,
        expect_id: u32,
        timeout: Option<Duration>,
    ) -> anyhow::Result<pb::protocol::WearPacket> {
        let (_seq, frame) = self.build_frame(channel, op, payload).await?;
        let (tx, rx) = oneshot::channel();
        self.pending_proto.insert((expect_type, expect_id), tx);
        let _guard = self.send_lock.lock().await;
        self.send_fragments(frame).await?;
        Ok(tokio::time::timeout(timeout.unwrap_or(REQ_TIMEOUT), rx).await??)
    }

    pub fn subscribe_proto(
        &self,
        expect_type: u32,
        callback: Arc<dyn Fn(pb::protocol::WearPacket) + Send + Sync>,
    ) {
        self.proto_subscribers
            .entry(expect_type)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub async fn wait_proto(
        &self,
        expect_type: u32,
        expect_id: u32,
        timeout: Option<Duration>,
    ) -> anyhow::Result<pb::protocol::WearPacket> {
        let (tx, rx) = oneshot::channel();
        let proto_key: ProtoKey = (expect_type, expect_id);

        match self.pending_proto.entry(proto_key) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                bail!(
                    "Another request is already waiting for ProtoKey {:?}",
                    proto_key
                );
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(tx);
            }
        }

        log::debug!(
            "[MiWearDevice.wait_proto] Waiting for ProtoKey ({}, {})",
            expect_type,
            expect_id
        );

        match tokio::time::timeout(timeout.unwrap_or(REQ_TIMEOUT), rx).await {
            Ok(Ok(packet)) => {
                log::debug!(
                    "[MiWearDevice.wait_proto] Received ProtoKey ({}, {})",
                    expect_type,
                    expect_id
                );

                Ok(packet)
            }
            Ok(Err(e)) => {
                self.pending_proto.remove(&proto_key);
                Err(anyhow!(
                    "Failed to receive proto: sender was dropped for ProtoKey ({}, {}). Error: {}",
                    expect_type,
                    expect_id,
                    e
                ))
            }
            Err(_) => {
                self.pending_proto.remove(&proto_key);
                Err(anyhow!(
                    "Timeout waiting for ProtoKey ({}, {}) (大狗大狗叫叫叫，你的资源疑似没装上。试着重启一下手环？)",
                    expect_type,
                    expect_id
                ))
            }
        }
    }

    pub(super) async fn build_frame(
        &self,
        channel: Channel,
        op: OpCode,
        payload: &[u8],
    ) -> anyhow::Result<(u8, Vec<u8>)> {
        let mut pdata = payload.to_vec();

        if op == OpCode::Encrypted {
            let keys = { self.state.read().await.sec_keys.clone().unwrap() };

            let enc_key = crate::tools::vec_to_array_16_opt(&keys.enc_key).unwrap();

            pdata = crate::crypto::aesctr::aes128_ctr_crypt(&enc_key, &enc_key, &pdata);
        }

        let seq = self
            .seq
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(if v == 255 { 0 } else { v + 1 })
            })
            .unwrap();
        let frame =
            packet::MiWearPacket::new_data(seq, channel, op, &std::borrow::Cow::Borrowed(&pdata))
                .encode();
        Ok((seq, frame))
    }

    pub fn take_seq_pending(&self, seq: u8) -> Option<oneshot::Sender<PacketData>> {
        self.pending_seq.remove(&seq).map(|(_, v)| v)
    }

    /* ───────────── 功能性函数 ───────────── */
    pub async fn start_hello(self: &Arc<Self>) -> anyhow::Result<()> {
        hello::hello_packet(self.clone()).await
    }

    pub async fn start_auth(self: &Arc<Self>, auth_key: String) -> anyhow::Result<()> {
        {
            let mut st = self.state.write().await;
            if st.sec_keys.is_some() {
                bail!("The device has been authenticated");
            }
            st.authkey = auth_key.clone();
        }
        log::info!("[MiWearDevice.start_auth] auth_key={}", &auth_key);
        let ret = auth::do_auth(self.clone()).await;

        let state = self.state.read().await.clone();
        crate::config::write(|c| {
            for dev in &mut c.paired_devices {
                if dev.name == state.name {
                    *dev = state.clone();
                }
            }
            c.current_device = Some(state.clone());
        });

        ret
    }

    pub async fn push_recv_data(self: &Arc<Self>, data: Vec<u8>) {
        self.recv_buffer.lock().await.extend(data);
    }

    pub async fn pop_bt_packet(self: &Arc<Self>) -> Option<Vec<u8>> {
        let mut recv: tokio::sync::MutexGuard<'_, Vec<u8>> = self.recv_buffer.lock().await;

        let buf: Vec<u8> = recv.clone();

        let mut result: Vec<u8> = Vec::new();

        let mut idx = 0;

        #[cfg(debug_assertions)]
        {
            log::info!(
                "[MiWearDevice] pop_bt_packet start recv_buffer(size: {})",
                recv.len()
            );
            let hex_recv = to_hex_string(&buf.clone());
            log::info!("[MiWearDevice] pop_bt_packet hex_recv: {}", hex_recv);
            log::info!("[MiWearDevice] pop_bt_packet while: {}", buf.len() - idx);
        }

        while buf.len() - idx > 8 {
            #[cfg(debug_assertions)]
            {
                log::info!("[MiWearDevice] pop_bt_packet while: {}", buf.len() - idx);
            }

            if buf[idx..].starts_with(&[0xA5, 0xA5]) {
                let len = u16::from_le_bytes(buf[idx + 4..idx + 6].try_into().unwrap()) as usize;
                let end_idx = idx + len + 8;

                #[cfg(debug_assertions)]
                {
                    log::info!("[MiWearDevice] pop_bt_packet idx: {}", idx);
                    log::info!("[MiWearDevice] pop_bt_packet packet len: {}", len);
                    log::info!("[MiWearDevice] pop_bt_packet packet end: {}", end_idx);
                    log::info!(
                        "[MiWearDevice] pop_bt_packet packet succ: {}",
                        buf.len() >= end_idx
                    );
                    log::info!("[MiWearDevice] pop_bt_packet buf len: {}", buf.len());
                }

                if buf.len() >= end_idx {
                    result.extend(buf[idx..end_idx].to_vec());
                    idx = end_idx;

                    #[cfg(debug_assertions)]
                    {
                        let hex = to_hex_string(&result.clone());

                        log::info!("[MiWearDevice] pop_bt_packet packet: {}", hex);
                        log::info!("[MiWearDevice] pop_bt_packet idx: {}", idx);
                    }
                } else {
                    break;
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    log::info!("[MiWearDevice] pop_bt_packet skip: {}", buf.len() - idx);
                }

                idx += 1;
            }
        }

        if idx != 0 {
            recv.drain(0..=idx - 1);

            #[cfg(debug_assertions)]
            {
                log::info!(
                    "[MiWearDevice] pop_bt_packet end recv_buffer(size: {})",
                    recv.len()
                );
            }
        }

        if result.len() > 0 {
            return Some(result);
        }

        None
    }

    pub async fn get_codename(self: &Arc<Self>) -> anyhow::Result<String> {
        {
            let st = self.state.read().await;
            if !st.codename.is_empty() {
                return Ok(st.codename.clone());
            }
        }

        let model = system::system_get_device_info(self.clone()).await?.model;
        let provider = crate::community::get_provider("official")
            .await
            .context("Official Provider not found")?;
        if provider.as_any().is::<OfficialProvider>() {
            let ptr = Arc::into_raw(provider) as *const OfficialProvider;

            // SAFETY: 名为official的Provider类型一定是OfficialProvider
            // 因此强行转换是安全的
            let official_provider = unsafe { Arc::from_raw(ptr) };

            let model_map = official_provider.get_device_map().await?;

            if let Some(dev) = model_map.get(&model) {
                let codename = dev.codename.clone();
                self.update_codename(&codename).await;
                return Ok(codename);
            }

            let name = self.state.read().await.name.clone();

            for dev in model_map.values() {
                if name.starts_with(&dev.name) {
                    let codename = dev.codename.clone();
                    self.update_codename(&codename).await;
                    return Ok(codename);
                }
            }

            return Err(anyhow!("Device not found"));
        }

        Err(anyhow!("Provider error"))
    }

    async fn update_codename(&self, codename: &str) {
        {
            let mut st = self.state.write().await;
            st.codename = codename.to_string();
        }
        let state = self.state.read().await.clone();
        crate::config::write(|c| {
            if let Some(dev) = c.current_device.as_mut() {
                if dev.addr == state.addr {
                    *dev = state.clone();
                }
            }
            for dev in &mut c.paired_devices {
                if dev.addr == state.addr {
                    *dev = state.clone();
                }
            }
        });
    }
}

/* ─────────────────────────────────────────────────────────── */

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MiWearBleCharaUuid {
    pub recv: Uuid,
    pub sent: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityKeys {
    pub enc_key: Vec<u8>,
    pub dec_key: Vec<u8>,
    pub enc_nonce: Vec<u8>,
    pub dec_nonce: Vec<u8>,
}

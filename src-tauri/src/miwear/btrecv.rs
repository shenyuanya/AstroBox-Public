use std::{io::Cursor};
use std::sync::Arc;
use prost::Message;
use crate::{
    miwear::packet::{Channel, Mi __OPENSOURCE__DELETED__, OpCode, PacketData, PktType},
    pb::{self},
    tools::{hex_stream_to_bytes, to_hex_string},
};
use super::device::MiWearDevice;

pub async fn handle_bt_packet(device: Arc<MiWearDevice>, data: Vec<u8>) {
    let data_str = to_hex_string(&data);

    #[cfg(debug_assertions)] {
        log::info!("[MiWearBTRecv] Recv data from miwear: {}", &data_str);
    }

    /* ────────── Hello 阶段帧 ────────── */
    if data_str.starts_with("badcfe") {
        super::device::hello::session_config_packet(device.clone())
            .await
            .unwrap();
        return;
    }

    /* ────────── MiWear Packet 解析 ────────── */
    match Mi __OPENSOURCE__DELETED__::parse_all(&data) {
        Ok(mipkts) => {
            for mipkt in mipkts {
                log::info!(
                    "[MiWearBTRecv] RX seq={} type={:?} (waiting for {:?})",
                    mipkt.seq,
                    mipkt.pkt_type,
                    device
                        .pending_seq
                        .iter()
                        .map(|e| *e.key())
                        .collect::<Vec<_>>()
                );

                match mipkt.pkt_type {
                    PktType::Ack => {
                        log::info!(
                            "[MiWearBTRecv] ACK Packet: packet with seq {} acked.",
                            mipkt.seq
                        );
                        if let Some((_, tx)) = device.pending_ack.remove(&()) {
                            let _ = tx.send(()); 
                            log::info!(
                                "[MiWearBTRecv] BTRecv: Matched ACK seq {} -> delivered",
                                mipkt.seq
                            );
                        }
                    }
                    PktType::SessionConfig => {
                        log::info!("[MiWearBTRecv] SessionConfig Packet: ignored.");
                    }
                    PktType::Data => {
                        // 拆 Data Frame
                        let payload = mipkt.data_fields().unwrap();

                        log::info!(
                            "[MiWearBTRecv] Data Packet: channel={:?}",
                            payload.channel
                        );

                        let mut content  = payload.data.to_vec();

                        if payload.opcode == OpCode::Encrypted {
                            let keys = { device.state.read().await.sec_keys.clone().unwrap() };
                            let dec_key = crate::tools::vec_to_array_16_opt(&keys.dec_key).unwrap();
                            content = crate::crypto::aesctr::aes128_ctr_crypt(
                                &dec_key, &dec_key, &content,
                            );
                        }                        

                        let pkt_data: PacketData = match payload.channel {
                            Channel::Pb => {
                                match  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::decode(Cursor::new(&content)) {
                                    Ok(packet) => {
                                        #[cfg(debug_assertions)] {
                                            log::info!(
                                                "[MiWearBTRecv] TYPE=Protobuf ID={} TYPE={:?}",
                                                packet.__OPENSOURCE__DELETED__,
                                                packet.__OPENSOURCE__DELETED__
                                            );

                                            log::info!(
                                                "[MiWearBTRecv] Protobuf: {}",
                                                serde_json::to_string(&packet).unwrap()
                                            );
                                        }

                                        PacketData::PROTOBUF(packet)
                                    }
                                    Err(e) => {
                                        log::error!("Error parsing packet to protobuf: {}", e);
                                        return;
                                    }
                                }
                            }
                            Channel::Mass | Channel::FileFitness => {
                                PacketData::DATA(content)
                            }
                            Channel::NetWork => {
                                PacketData::NETWORK(content)
                            }
                            _ => {
                                PacketData::UNSUPPORT(content)
                            }
                        };

                        if let Some(tx) = device.take_seq_pending(mipkt.seq) {
                            let _ = tx.send(pkt_data.clone());
                            log::info!("[MiWearBTRecv] BTRecv: TxRequestResultSent");
                        }

                        if let PacketData::PROTOBUF(ref wp) = pkt_data.clone() {
                            let key = (wp.__OPENSOURCE__DELETED__ as u32, wp.id);
                            if let Some((_, sender)) = device.pending_proto.remove(&key) {
                                let _ = sender.send(wp.clone());
                                log::info!("[MiWearBTRecv] BTRecv: Matched ProtoKey({:?}) -> delivered", key);
                            }
                            if let Some(subs) = device.proto_subscribers.get(&(wp.__OPENSOURCE__DELETED__ as u32)) {
                                for cb in subs.value().iter() {
                                    let pkt = wp.clone();
                                    cb(pkt.clone());
                                }
                            }
                        }

                        if let PacketData::NETWORK(ref network_packet) = pkt_data.clone() {
                            log::info!("[MiWearBTRecv] BTRecv: Received TunPacket(size: {})", network_packet.len());
                            
                            let mut dhcp_process = false;

                            super::network_stack::dhcp::process_dhcp(device.clone(), network_packet, &mut dhcp_process).await;

                            if !dhcp_process {
                                let tx_guard = device.network_tx.lock().await;
                                if let Some(tx) = tx_guard.as_ref() {
                                    if let Err(e) = tx.send(network_packet.clone()).await {
                                        log::error!("[MiWearBTRecv] Failed to send packet to network stack: {}", e);
                                    }
                                } else {
                                    log::warn!("[MiWearBTRecv] Received network packet, but stack is not running.");
                                }
                            }
                        }

                        if let PacketData::UNSUPPORT(ref unk) = pkt_data.clone() {
                            #[cfg(debug_assertions)] {
                                log::info!("[MiWearBTRecv] BTRecv: Received UnsupportPacket: {}", to_hex_string(unk));
                            }
                        }

                        let _guard = device.send_lock.lock().await;
                        // 回 ACK
                        device
                            .send(
                                hex_stream_to_bytes(&format!("a5a501{:02x}00000000", mipkt.seq))
                                    .unwrap(),
                            )
                            .await
                            .unwrap();

                        log::info!("[MiWearBTRecv] BTRecv: Data process finished");
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Error parsing packet: {}", e);
        }
    }
}

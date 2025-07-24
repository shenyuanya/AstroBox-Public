use super::MiWearDevice;
use crate::{
    miwear::packet::{Channel, OpCode},
    pb::{self},
};
use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use once_cell::sync::Lazy;
use packet::{MassDataType, MassPacket};
use prost::Message;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod packet;

#[derive(Clone)]
struct ResumeState {
    device_addr: String,
    mass_id: Vec<u8>,
    current_part: u16,
}

static RESUME_STATE: Lazy<Mutex<Option<ResumeState>>> = Lazy::new(|| Mutex::new(None));

pub async fn clear_resume_state() {
    *RESUME_STATE.lock().await = None;
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMassCallbackData {
    pub progress: f32,
    pub total_parts: u16,
    pub current_part_num: u16,
    pub actual_data_payload_len: usize,
}

pub async fn send_mass<F>(
    device: &Arc<MiWearDevice>,
    file_data: Vec<u8>,
    data_type: MassDataType,
    progress_cb: F,
) -> Result<()>
where
    F: Fn(SendMassCallbackData) + Send + Sync,
{
    let file_md5_for_prepare = crate::tools::calc_md5(&file_data);

    let mut disconnect_rx = crate::miwear::subscribe_disconnect();

    let device_addr = device.state.read().await.addr.clone();
    let mut start_part: u16 = 1;
    {
        let mut guard = RESUME_STATE.lock().await;
        match guard.as_mut() {
            Some(state)
                if state.mass_id == file_md5_for_prepare && state.device_addr == device_addr =>
            {
                start_part = state.current_part;
            }
            _ => {
                *guard = Some(ResumeState {
                    device_addr,
                    mass_id: file_md5_for_prepare.clone(),
                    current_part: 1,
                });
            }
        }
    }

    let prepare_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_mass_prepare_request(data_type, &file_md5_for_prepare, file_data.len())
                .encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::Mass as u32,
             __OPENSOURCE__DELETED__mass::MassId::Prepare as u32,
            None,
        )
        .await
        .context("Mass prepare request_proto call failed")?;

    match prepare_ret
        .payload
        .and_then(|pl| match pl {
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Mass(m) => Some(m),
            _ => None,
        })
    {
        Some(mass) => {
            let prepare_resp = match mass.payload {
                Some( __OPENSOURCE__DELETED__mass::Payload::PrepareResponse(resp)) => resp,
                _ => bail!("Prepare response field was not present in the received protobuf."),
            };

            if prepare_resp.prepare_status !=  __OPENSOURCE__DELETED__PrepareStatus::Ready as i32 {
                bail!("Mass data prepare was not READY.");
            }

            log::info!(
                "[MiWearMassSend] Prepare response READY. Expected slice length: {}",
                prepare_resp.expected_slice_length()
            );

            // (comp_data | type | md5 | len | data | crc32)
            let mass_inner_payload_with_crc32 =
                MassPacket::build(file_data, data_type)?.encode_with_crc32();
            log::info!(
                "[MiWearMassSend] Encoded MassInnerData with CRC32, total length: {}",
                mass_inner_payload_with_crc32.len()
            );

            let mi __OPENSOURCE__DELETED___body_max_len = prepare_resp.expected_slice_length() as usize;
            if mi __OPENSOURCE__DELETED___body_max_len == 0 {
                bail!("Device reported expected_slice_length of 0, cannot proceed.");
            }

            // Mi __OPENSOURCE__DELETED__ Body = Channel(1) | Op(1) | blocks_num(2) | resume_block(2) | MassFragment
            let mass_fragment_max_len = mi __OPENSOURCE__DELETED___body_max_len.saturating_sub(1 + 1 + 2 + 2);
            if mass_fragment_max_len == 0 {
                bail!(
                    "Calculated mass_fragment_max_len is 0. Device limit ({}) is too small.",
                    mi __OPENSOURCE__DELETED___body_max_len
                );
            }

            let total_parts = (mass_inner_payload_with_crc32.len() as f32
                / mass_fragment_max_len as f32)
                .ceil() as u16;
            if total_parts == 0 && !mass_inner_payload_with_crc32.is_empty() {
                bail!("Calculated total_parts is 0 for non-empty payload.");
            }
            if total_parts == 0 && mass_inner_payload_with_crc32.is_empty() {
                log::info!(
                    "[MiWearMassSend] MassInnerData is empty, sending 0 parts (or 1 empty part)."
                );
            }

            log::info!(
                "[MiWearMassSend] Chunking: total_parts={}, MassInnerPayload len={}, MassFragment max_len={}",
                total_parts,
                mass_inner_payload_with_crc32.len(),
                mass_fragment_max_len
            );

            let _send_lock = device.send_lock.lock().await;
            *(device.is_sending_mass.lock().await) = true;

            for i in (start_part - 1)..total_parts {
                let current_part_num = i + 1; // 1-indexed

                // slice indices
                let start_index = i as usize * mass_fragment_max_len;
                let end_index = std::cmp::min(
                    start_index + mass_fragment_max_len,
                    mass_inner_payload_with_crc32.len(),
                );
                let mass_fragment_slice = &mass_inner_payload_with_crc32[start_index..end_index];

                // ActualDataPayload = blocks_num | resume_block | MassFragment
                let mut actual_data_payload = Vec::with_capacity(4 + mass_fragment_slice.len());
                actual_data_payload
                    .write_u16::<LittleEndian>(total_parts)
                    .unwrap();
                actual_data_payload
                    .write_u16::<LittleEndian>(current_part_num)
                    .unwrap();
                actual_data_payload.extend_from_slice(mass_fragment_slice);

                // progress callback
                progress_cb(SendMassCallbackData {
                    progress: current_part_num as f32 / total_parts as f32,
                    total_parts,
                    current_part_num,
                    actual_data_payload_len: actual_data_payload.len(),
                });

                if disconnect_rx.try_recv().is_ok() {
                    bail!("Device disconnected during mass send");
                }

                let ack_rx = device
                    .send_miwear_pkt_register_ack_unlocked(
                        Channel::Mass,
                        OpCode::Plain,
                        &actual_data_payload,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send mass data part {}/{}",
                            current_part_num, total_parts
                        )
                    })?;

                let total_parts_clone = total_parts;
                tokio::spawn(async move {
                    let mut ack_disconnect_rx = crate::miwear::subscribe_disconnect();
                    tokio::select! {
                        _ = ack_rx => {
                            let mut guard = RESUME_STATE.lock().await;
                            if let Some(state) = guard.as_mut() {
                                state.current_part = current_part_num + 1;
                                if state.current_part > total_parts_clone {
                                    *guard = None;
                                }
                            }
                        }
                        _ = ack_disconnect_rx.recv() => {
                            log::warn!("Device disconnected before ACK of part {}", current_part_num);
                        }
                    }
                });
            }

            *(device.is_sending_mass.lock().await) = false;
            log::info!(
                "[MiWearMassSend] All mass data parts queued; returning without waiting for ACKs."
            );
            Ok(())
        }
        None => bail!("Mass field was not present in the received protobuf."),
    }
}

fn build_mass_prepare_request(
    data_type: MassDataType,
    file_md5: &Vec<u8>,
    file_length: usize,
) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let mass_payload =  __OPENSOURCE__DELETED__PrepareRequest {
        data_type: data_type as u32,
        data_id: file_md5.to_vec(),
        data_length: file_length as u32,
        support_compress_mode: None,
    };

    let mass_pkt =  __OPENSOURCE__DELETED__Mass {
        payload: Some( __OPENSOURCE__DELETED__mass::Payload::PrepareRequest(mass_payload)),
    };

     __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        r#type:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::Mass as i32,
        id:  __OPENSOURCE__DELETED__mass::MassId::Prepare as u32,
        payload: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Mass(mass_pkt)),
    }
}
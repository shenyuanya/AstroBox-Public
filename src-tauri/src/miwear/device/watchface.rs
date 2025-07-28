use crate::{
    miwear::packet::{Channel, OpCode},
    pb::{self},
};
use anyhow::{bail, Result};
use prost::Message;
use std::{sync::Arc, time::Duration};
use serde::{Deserialize, Serialize};

use super::{
    mass::{packet::MassDataType, SendMassCallbackData},
    MiWearDevice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchfaceInfo {
    pub id: String,
    pub name: String,
    pub is_current: bool,
    pub can_remove: Option<bool>,
    pub version_code: Option<u64>,
    pub can_edit: Option<bool>,
    pub background_color: String,
    pub background_image: String,
    pub style: String,
    pub background_image_list: Vec<String>,
}

pub async fn install_watchface<F>(
    device: Arc<MiWearDevice>,
    file_path: &String,
    new_watchface_id: Option<&Vec<u8>>,
    id: &String,
    progress_cb: F,
) -> Result<()>
where
    F: Fn(SendMassCallbackData) + Send + Sync,
{
    let mut file_data = crate::fs::read_file_cross_platform(file_path).await?;

    if let Some(id_bytes) = new_watchface_id {
        for (i, &byte) in id_bytes.iter().enumerate() {
            file_data[0x28 + i] = byte;
        }
    }

    let watchface_install_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_watchface_install_request(id, file_data.len()).encode_to_vec(),
            pb::protocol::wear_packet::Type::WatchFace as u32,
            pb::protocol::watch_face::WatchFaceId::PrepareInstallWatchFace as u32,
            None,
        )
        .await?;

    match watchface_install_ret.__OPENSOURCE_DELETED__.unwrap() {
        pb::protocol::wear_packet::Payload::WatchFace(watch_face) => match watch_face.__OPENSOURCE_DELETED__.unwrap() {
            pb::protocol::watch_face::Payload::PrepareStatus(prepare_status) => {
                if prepare_status == pb::protocol::PrepareStatus::Ready as i32 {
                    super::mass::send_mass(
                        &device,
                        file_data,
                        MassDataType::WATCHFACE,
                        progress_cb,
                    )
                    .await?;

                    device
                        .wait_proto(
                            pb::protocol::wear_packet::Type::WatchFace as u32,
                            pb::protocol::watch_face::WatchFaceId::ReportInstallResult as u32,
                            Some(Duration::from_secs(10)),
                        )
                        .await?;
                } else {
                    #[cfg(debug_assertions)] {
                        bail!(
                            "Prepare not READY! Full resp: {}",
                            serde_json::to_string(&prepare_status).unwrap()
                        );
                    }
                    #[cfg(not(debug_assertions))] {
                        bail!(
                            "Prepare not READY! Error Info={}",
                            super::error::get_prepare_error_info(prepare_status)
                        );
                    }
                }
            }
            _ => {
                bail!("Prepare status doesn't exsist!");
            }
        },
        _ => {
            bail!("WatchFace doesn't exsist!");
        }
    }

    Ok(())
}

pub async fn get_watchface_list(device: Arc<MiWearDevice>) -> Result<Vec<WatchfaceInfo>> {
    let watchface_installed_ret: pb::protocol::WearPacket = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_watchface_get_installed().encode_to_vec(),
            pb::protocol::wear_packet::Type::WatchFace as u32,
            pb::protocol::watch_face::WatchFaceId::GetInstalledList as u32,
            None,
        )
        .await?;

    if let Some(payload) = watchface_installed_ret.__OPENSOURCE_DELETED__
    {
        match payload {
            pb::protocol::wear_packet::Payload::WatchFace(watchface) => {
                if let Some(watchface_payload) = watchface.__OPENSOURCE_DELETED__
                {
                    match watchface_payload {
                        pb::protocol::watch_face::Payload::WatchFaceList(list) => {
                            let mut result = Vec::new();

                            list.__OPENSOURCE_DELETED__.iter().for_each(|item| {
                                result.push(WatchfaceInfo {
                                    id: item.__OPENSOURCE_DELETED__.clone(),
                                    name: item.__OPENSOURCE_DELETED__.clone(),
                                    is_current: item.__OPENSOURCE_DELETED__,
                                    can_remove: item.__OPENSOURCE_DELETED__,
                                    version_code: item.__OPENSOURCE_DELETED__,
                                    can_edit: item.__OPENSOURCE_DELETED__,
                                    background_color: item.__OPENSOURCE_DELETED__.clone(),
                                    background_image: item.__OPENSOURCE_DELETED__.clone(),
                                    style: item.__OPENSOURCE_DELETED__.clone(),
                                    background_image_list: item.__OPENSOURCE_DELETED__.clone(),
                                });
                            });

                            return Ok(result);
                        }
                        _ => {
                            bail!("List doesn't exsist!");
                        }
                    }
                }
            }
            _ => {
                bail!("Watchface doesn't exsist!");
            }
        }
    }

    bail!("Packet doesn't exsist!");
}

pub async fn uninstall_watchface(device: Arc<MiWearDevice>, watchface: WatchfaceInfo) -> Result<()> {
    device
        .send_miwear_pkt(
            Channel::Pb,
            OpCode::Encrypted,
            &build_watchface_uninstall(watchface).encode_to_vec(),
        )
        .await?;

    Ok(())
}

pub async fn set_watchface(device: Arc<MiWearDevice>, watchface: WatchfaceInfo) -> Result<()> {
    device
        .send_miwear_pkt(
            Channel::Pb,
            OpCode::Encrypted,
            &build_set_watchface(watchface).encode_to_vec(),
        )
        .await?;

    Ok(())
}

fn build_watchface_install_request(id: &String, package_size: usize) -> pb::protocol::WearPacket {
    let prepare_info = pb::protocol::PrepareInfo {
        __OPENSOURCE_DELETED__: id.to_string(),
        __OPENSOURCE_DELETED__: package_size as u32,
        __OPENSOURCE_DELETED__: Some(65536),
        __OPENSOURCE_DELETED__: None,
        __OPENSOURCE_DELETED__: None,
    };

    let pkt_payload = pb::protocol::WatchFace {
        __OPENSOURCE_DELETED__: Some(pb::protocol::watch_face::Payload::PrepareInfo(prepare_info)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::WatchFace as i32,
        __OPENSOURCE_DELETED__: pb::protocol::watch_face::WatchFaceId::PrepareInstallWatchFace as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::WatchFace(pkt_payload)),
    };

    pkt
}

fn build_watchface_get_installed() -> pb::protocol::WearPacket {
    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::WatchFace as i32,
        __OPENSOURCE_DELETED__: pb::protocol::watch_face::WatchFaceId::GetInstalledList as u32,
        __OPENSOURCE_DELETED__: None,
    };

    pkt
}

fn build_watchface_uninstall(watchface: WatchfaceInfo) -> pb::protocol::WearPacket {
    let pkt_payload = pb::protocol::WatchFace {
        __OPENSOURCE_DELETED__: Some(pb::protocol::watch_face::Payload::Id(watchface.id)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::WatchFace as i32,
        __OPENSOURCE_DELETED__: pb::protocol::watch_face::WatchFaceId::RemoveWatchFace as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::WatchFace(pkt_payload)),
    };

    pkt
}

fn build_set_watchface(watchface: WatchfaceInfo) -> pb::protocol::WearPacket {
    let pkt_payload = pb::protocol::WatchFace {
        __OPENSOURCE_DELETED__: Some(pb::protocol::watch_face::Payload::Id(watchface.id)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::WatchFace as i32,
        __OPENSOURCE_DELETED__: pb::protocol::watch_face::WatchFaceId::SetWatchFace as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::WatchFace(pkt_payload)),
    };

    pkt
}
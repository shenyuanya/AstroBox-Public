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
    id: &String,
    progress_cb: F,
) -> Result<()>
where
    F: Fn(SendMassCallbackData) + Send + Sync,
{
    let file_data = crate::fs::read_file_cross_platform(file_path).await?;

    let watchface_install_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_watchface_install_request(id, file_data.len()).encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as u32,
             __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::PrepareInstallWatchFace as u32,
            None,
        )
        .await?;

    match watchface_install_ret.__OPENSOURCE__DELETED__.unwrap() {
         __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::WatchFace(watch_face) => match watch_face.__OPENSOURCE__DELETED__.unwrap() {
             __OPENSOURCE__DELETED__watch_face::Payload::PrepareStatus(prepare_status) => {
                if prepare_status ==  __OPENSOURCE__DELETED__PrepareStatus::Ready as i32 {
                    super::mass::send_mass(
                        &device,
                        file_data,
                        MassDataType::WATCHFACE,
                        progress_cb,
                    )
                    .await?;

                    device
                        .wait_proto(
                             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as u32,
                             __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::ReportInstallResult as u32,
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
                            "Prepare not READY!"
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
    let watchface_installed_ret:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_watchface_get_installed().encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as u32,
             __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::GetInstalledList as u32,
            None,
        )
        .await?;

    if let Some(payload) = watchface_installed_ret.__OPENSOURCE__DELETED__
    {
        match payload {
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::WatchFace(watchface) => {
                if let Some(watchface_payload) = watchface.__OPENSOURCE__DELETED__
                {
                    match watchface_payload {
                         __OPENSOURCE__DELETED__watch_face::Payload::WatchFaceList(list) => {
                            let mut result = Vec::new();

                            list.__OPENSOURCE__DELETED__.iter().for_each(|item| {
                                result.push(WatchfaceInfo {
                                    id: item.__OPENSOURCE__DELETED__.clone(),
                                    name: item.__OPENSOURCE__DELETED__.clone(),
                                    is_current: item.__OPENSOURCE__DELETED__,
                                    can_remove: item.__OPENSOURCE__DELETED__,
                                    version_code: item.__OPENSOURCE__DELETED__,
                                    can_edit: item.__OPENSOURCE__DELETED__,
                                    background_color: item.__OPENSOURCE__DELETED__.clone(),
                                    background_image: item.__OPENSOURCE__DELETED__.clone(),
                                    style: item.__OPENSOURCE__DELETED__.clone(),
                                    background_image_list: item.__OPENSOURCE__DELETED__.clone(),
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

fn build_watchface_install_request(id: &String, package_size: usize) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let prepare_info =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__: id.to_string(),
        __OPENSOURCE__DELETED__: package_size as u32,
        __OPENSOURCE__DELETED__: Some(65536),
        __OPENSOURCE__DELETED__: None,
        __OPENSOURCE__DELETED__: None,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__WatchFace {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__watch_face::Payload:: __OPENSOURCE__DELETED__(prepare_info)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::PrepareInstallWatchFace as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::WatchFace(pkt_payload)),
    };

    pkt
}

fn build_watchface_get_installed() ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::GetInstalledList as u32,
        __OPENSOURCE__DELETED__: None,
    };

    pkt
}

fn build_watchface_uninstall(watchface: WatchfaceInfo) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let pkt_payload =  __OPENSOURCE__DELETED__WatchFace {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__watch_face::Payload::Id(watchface.id)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::RemoveWatchFace as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::WatchFace(pkt_payload)),
    };

    pkt
}

fn build_set_watchface(watchface: WatchfaceInfo) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let pkt_payload =  __OPENSOURCE__DELETED__WatchFace {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__watch_face::Payload::Id(watchface.id)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::WatchFace as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__watch_face:: __OPENSOURCE__DELETED__::SetWatchFace as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::WatchFace(pkt_payload)),
    };

    pkt
}
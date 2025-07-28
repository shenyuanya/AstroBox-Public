use crate::{
    miwear::packet::{Channel, OpCode},
    pb::{self},
};
use anyhow::{bail, Result};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc, time::Duration};

use super::{
    mass::{packet::MassDataType, SendMassCallbackData},
    MiWearDevice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub package_name: String,
    pub fingerprint: Vec<u8>,
    pub version_code: u32,
    pub can_remove: bool,
    pub app_name: String,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Connected = 1,
    Disconnected = 2,
}

pub async fn install_app<F>(
    device: Arc<MiWearDevice>,
    file_path: &String,
    package_name: &String,
    version_code: u32,
    progress_cb: F,
) -> Result<()>
where
    F: Fn(SendMassCallbackData) + Send + Sync,
{
    let file_data = crate::fs::read_file_cross_platform(file_path).await?;

    let thirdparty_app_install_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_thirdparty_app_install_request(package_name, version_code, file_data.len())
                .encode_to_vec(),
            pb::protocol::wear_packet::Type::ThirdpartyApp as u32,
            pb::protocol::thirdparty_app::ThirdpartyAppId::PrepareInstallApp as u32,
            None,
        )
        .await?;

    match thirdparty_app_install_ret.__OPENSOURCE_DELETED__.unwrap() {
        pb::protocol::wear_packet::Payload::ThirdpartyApp(thirdparty_app) => {
            match thirdparty_app.__OPENSOURCE_DELETED__.unwrap() {
                pb::protocol::thirdparty_app::Payload::InstallResponse(response) => {
                    if response.__OPENSOURCE_DELETED__ == pb::protocol::PrepareStatus::Ready as i32
                    {
                        super::mass::send_mass(
                            &device,
                            file_data,
                            MassDataType::ThirdpartyApp,
                            progress_cb,
                        )
                        .await?;

                        device
                            .wait_proto(
                                pb::protocol::wear_packet::Type::ThirdpartyApp as u32,
                                pb::protocol::thirdparty_app::ThirdpartyAppId::ReportInstallResult
                                    as u32,
                                Some(Duration::from_secs(30)),
                            )
                            .await?;
                    } else {
                        #[cfg(debug_assertions)] {
                            bail!(
                                "Prepare not READY! Full resp: {}",
                                serde_json::to_string(&response).unwrap()
                            );
                        }
                        #[cfg(not(debug_assertions))] {
                            bail!(
                                "Prepare not READY! Error Info={}",
                                super::error::get_prepare_error_info(response.__OPENSOURCE_DELETED__ as i32)
                            );
                        }
                    }
                }
                _ => {
                    bail!("Prepare response doesn't exsist!");
                }
            };
        }
        _ => {
            bail!("App doesn't exsist!");
        }
    };

    Ok(())
}

pub async fn get_app_list(device: Arc<MiWearDevice>) -> Result<Vec<AppInfo>> {
    let thirdparty_app_installed_ret: pb::protocol::WearPacket = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_thirdparty_app_get_installed().encode_to_vec(),
            pb::protocol::wear_packet::Type::ThirdpartyApp as u32,
            pb::protocol::thirdparty_app::ThirdpartyAppId::GetInstalledList as u32,
            None,
        )
        .await?;

    if let Some(payload) = thirdparty_app_installed_ret.__OPENSOURCE_DELETED__
    {
        match payload {
            pb::protocol::wear_packet::Payload::ThirdpartyApp(thirdparty_app) => {
                if let Some(thirdparty_app_payload) = thirdparty_app.__OPENSOURCE_DELETED__
                {
                    match thirdparty_app_payload {
                        pb::protocol::thirdparty_app::Payload::AppItemList(list) => {
                            let mut result = Vec::new();

                            list.__OPENSOURCE_DELETED__.iter().for_each(|item| {
                                result.push(AppInfo {
                                    package_name: item.__OPENSOURCE_DELETED__.clone(),
                                    fingerprint: item.__OPENSOURCE_DELETED__.clone(),
                                    version_code: item.__OPENSOURCE_DELETED__,
                                    can_remove: item.__OPENSOURCE_DELETED__,
                                    app_name: item.__OPENSOURCE_DELETED__.clone(),
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
                bail!("App doesn't exsist!");
            }
        }
    }

    bail!("Packet doesn't exsist!");
}

pub async fn send_inter_packet(
    device: Arc<MiWearDevice>,
    package_name: &String,
    data: &String,
) -> Result<()> {
    if let Some(app) = device.app_info_table.get(package_name) {
        device
            .send_miwear_pkt(
                Channel::Pb,
                OpCode::Encrypted,
                &build_thirdparty_app_msg_content(app.value().clone(), data)
                    .encode_to_vec(),
            )
            .await?;
        Ok(())
    } else {
        bail!("AppInfo not found for {}", package_name)
    }
}

pub async fn launch_app(device: Arc<MiWearDevice>, app: AppInfo, page: &String) -> Result<()> {
    device
        .send_miwear_pkt(
            Channel::Pb,
            OpCode::Encrypted,
            &build_thirdparty_app_launch(app, page).encode_to_vec(),
        )
        .await?;

    Ok(())
}

pub async fn uninstall_app(device: Arc<MiWearDevice>, app: AppInfo) -> Result<()> {
    device
        .send_miwear_pkt(
            Channel::Pb,
            OpCode::Encrypted,
            &build_thirdparty_app_uninstall(app).encode_to_vec(),
        )
        .await?;

    Ok(())
}

pub fn on_thirdparty_app(device: Arc<MiWearDevice>, packet: pb::protocol::WearPacket) {
    if let Some(payload) = packet.__OPENSOURCE_DELETED__
    {
        if let pb::protocol::wear_packet::Payload::ThirdpartyApp(thirdparty_app) = payload {
            if let Some(thirdparty_app_payload) = thirdparty_app.__OPENSOURCE_DELETED__
            {
                match thirdparty_app_payload {
                    pb::protocol::thirdparty_app::Payload::BasicInfo(basic_info) => {
                        let app_info = AppInfo {
                            package_name: basic_info.__OPENSOURCE_DELETED__.clone(),
                            fingerprint: basic_info.__OPENSOURCE_DELETED__.clone(),
                            version_code: 0,
                            can_remove: false,
                            app_name: String::from(""),
                        };

                        device
                            .app_info_table
                            .insert(app_info.package_name.clone(), app_info.clone());

                        let pkt =
                            build_thirdparty_app_sync_status(app_info, AppStatus::Connected)
                                .encode_to_vec();

                        tokio::spawn(async move {
                            device
                                .send_miwear_pkt(Channel::Pb, OpCode::Encrypted, &pkt)
                                .await
                        });
                    }

                    pb::protocol::thirdparty_app::Payload::MessageContent(message_content) => {
                        tokio::spawn(async move {
                            crate::pluginsystem::with_plugin_manager_async(move |pm| {
                                for plug in pm.plugins.values_mut() {
                                    for event in &plug.js_env_data.event_listeners {
                                        if event.0
                                            == &("onQAICMessage_".to_string()
                                                + &message_content.__OPENSOURCE_DELETED__.__OPENSOURCE_DELETED__)
                                        {
                                            event
                                                .1
                                                .call(
                                                    &boa_engine::JsValue::undefined(),
                                                    &[boa_engine::JsValue::String(
                                                        boa_engine::JsString::from_str(
                                                            &String::from_utf8_lossy(
                                                                &message_content.__OPENSOURCE_DELETED__,
                                                            ),
                                                        )
                                                        .unwrap(),
                                                    )],
                                                    &mut plug.js_context,
                                                )
                                                .unwrap();
                                        }
                                    }
                                }
                            })
                            .await
                            .ok();
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

fn build_thirdparty_app_msg_content(app: AppInfo, data: &String) -> pb::protocol::WearPacket {

    let basic_info = pb::protocol::BasicInfo {
        __OPENSOURCE_DELETED__: app.package_name.clone(),
        __OPENSOURCE_DELETED__: app.fingerprint.clone(),
    };

    let message_content = pb::protocol::MessageContent {
        __OPENSOURCE_DELETED__: basic_info,
        __OPENSOURCE_DELETED__: data.clone().into(),
    };

    let pkt_payload = pb::protocol::ThirdpartyApp {
        __OPENSOURCE_DELETED__: Some(pb::protocol::thirdparty_app::Payload::MessageContent(message_content)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::SendPhoneMessage as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt

}

fn build_thirdparty_app_launch(app: AppInfo, page: &String) -> pb::protocol::WearPacket {
    let basic_info = pb::protocol::BasicInfo {
        __OPENSOURCE_DELETED__: app.package_name.clone(),
        __OPENSOURCE_DELETED__: app.fingerprint.clone(),
    };

    let launch_info = pb::protocol::LaunchInfo {
        __OPENSOURCE_DELETED__: basic_info,
        __OPENSOURCE_DELETED__: page.clone(),
    };

    let pkt_payload = pb::protocol::ThirdpartyApp {
        __OPENSOURCE_DELETED__: Some(pb::protocol::thirdparty_app::Payload::LaunchInfo(launch_info)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::LaunchApp as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

fn build_thirdparty_app_uninstall(app: AppInfo) -> pb::protocol::WearPacket {
    let basic_info = pb::protocol::BasicInfo {
        __OPENSOURCE_DELETED__: app.package_name.clone(),
        __OPENSOURCE_DELETED__: app.fingerprint.clone(),
    };

    let pkt_payload = pb::protocol::ThirdpartyApp {
        __OPENSOURCE_DELETED__: Some(pb::protocol::thirdparty_app::Payload::BasicInfo(basic_info)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::RemoveApp as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}


fn build_thirdparty_app_sync_status(app: AppInfo, status: AppStatus) -> pb::protocol::WearPacket {
    let basic_info = pb::protocol::BasicInfo {
        __OPENSOURCE_DELETED__: app.package_name.clone(),
        __OPENSOURCE_DELETED__: app.fingerprint.clone(),
    };

    let app_status = pb::protocol::PhoneAppStatus {
        __OPENSOURCE_DELETED__: basic_info,
        __OPENSOURCE_DELETED__: status as i32,
    };

    let pkt_payload = pb::protocol::ThirdpartyApp {
        __OPENSOURCE_DELETED__: Some(pb::protocol::thirdparty_app::Payload::AppStatus(app_status)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::SyncPhoneAppStatus as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

fn build_thirdparty_app_get_installed() -> pb::protocol::WearPacket {
    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::GetInstalledList as u32,
        __OPENSOURCE_DELETED__: None,
    };

    pkt
}

fn build_thirdparty_app_install_request(
    package_name: &String,
    version_code: u32,
    package_size: usize,
) -> pb::protocol::WearPacket {
    let install_req = pb::protocol::app_installer::Request {
        __OPENSOURCE_DELETED__: package_name.to_string(),
        __OPENSOURCE_DELETED__: version_code,
        __OPENSOURCE_DELETED__: package_size as u32,
    };

    let pkt_payload = pb::protocol::ThirdpartyApp {
        __OPENSOURCE_DELETED__: Some(pb::protocol::thirdparty_app::Payload::InstallRequest(install_req)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::ThirdpartyApp as i32,
        __OPENSOURCE_DELETED__: pb::protocol::thirdparty_app::ThirdpartyAppId::PrepareInstallApp as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

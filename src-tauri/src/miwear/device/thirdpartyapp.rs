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
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as u32,
             __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::PrepareInstallApp as u32,
            None,
        )
        .await?;

    match thirdparty_app_install_ret.__OPENSOURCE__DELETED__.unwrap() {
         __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(thirdparty_app) => {
            match thirdparty_app.__OPENSOURCE__DELETED__.unwrap() {
                 __OPENSOURCE__DELETED__thirdparty_app::Payload::InstallResponse(response) => {
                    if response.__OPENSOURCE__DELETED__ ==  __OPENSOURCE__DELETED__PrepareStatus::Ready as i32
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
                                 __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as u32,
                                 __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::ReportInstallResult
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
                                "Prepare not READY!"
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
    let thirdparty_app_installed_ret:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_thirdparty_app_get_installed().encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as u32,
             __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::GetInstalledList as u32,
            None,
        )
        .await?;

    if let Some(payload) = thirdparty_app_installed_ret.__OPENSOURCE__DELETED__
    {
        match payload {
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(thirdparty_app) => {
                if let Some(thirdparty_app_payload) = thirdparty_app.__OPENSOURCE__DELETED__
                {
                    match thirdparty_app_payload {
                         __OPENSOURCE__DELETED__thirdparty_app::Payload::AppItemList(list) => {
                            let mut result = Vec::new();

                            list.__OPENSOURCE__DELETED__.iter().for_each(|item| {
                                result.push(AppInfo {
                                    package_name: item.__OPENSOURCE__DELETED__.clone(),
                                    fingerprint: item.__OPENSOURCE__DELETED__.clone(),
                                    version_code: item.__OPENSOURCE__DELETED__,
                                    can_remove: item.__OPENSOURCE__DELETED__,
                                    app_name: item.__OPENSOURCE__DELETED__.clone(),
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

pub fn on_thirdparty_app(device: Arc<MiWearDevice>, packet:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__) {
    if let Some(payload) = packet.__OPENSOURCE__DELETED__
    {
        if let  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(thirdparty_app) = payload {
            if let Some(thirdparty_app_payload) = thirdparty_app.__OPENSOURCE__DELETED__
            {
                match thirdparty_app_payload {
                     __OPENSOURCE__DELETED__thirdparty_app::Payload::BasicInfo(basic_info) => {
                        let app_info = AppInfo {
                            package_name: basic_info.__OPENSOURCE__DELETED__.clone(),
                            fingerprint: basic_info.__OPENSOURCE__DELETED__.clone(),
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

                     __OPENSOURCE__DELETED__thirdparty_app::Payload::MessageContent(message_content) => {
                        tokio::spawn(async move {
                            crate::pluginsystem::with_plugin_manager_async(move |pm| {
                                for plug in pm.plugins.values_mut() {
                                    for event in &plug.js_env_data.event_listeners {
                                        if event.0
                                            == &("onQAICMessage_".to_string()
                                                + &message_content.__OPENSOURCE__DELETED__.__OPENSOURCE__DELETED__)
                                        {
                                            event
                                                .1
                                                .call(
                                                    &boa_engine::JsValue::undefined(),
                                                    &[boa_engine::JsValue::String(
                                                        boa_engine::JsString::from_str(
                                                            &String::from_utf8_lossy(
                                                                &message_content.__OPENSOURCE__DELETED__,
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

fn build_thirdparty_app_msg_content(app: AppInfo, data: &String) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {

    let basic_info =  __OPENSOURCE__DELETED__BasicInfo {
        __OPENSOURCE__DELETED__: app.package_name.clone(),
        __OPENSOURCE__DELETED__: app.fingerprint.clone(),
    };

    let message_content =  __OPENSOURCE__DELETED__MessageContent {
        __OPENSOURCE__DELETED__: basic_info,
        __OPENSOURCE__DELETED__: data.clone().into(),
    };

    let pkt_payload =  __OPENSOURCE__DELETED__ThirdpartyApp {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__thirdparty_app::Payload::MessageContent(message_content)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::SendPhoneMessage as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt

}

fn build_thirdparty_app_launch(app: AppInfo, page: &String) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let basic_info =  __OPENSOURCE__DELETED__BasicInfo {
        __OPENSOURCE__DELETED__: app.package_name.clone(),
        __OPENSOURCE__DELETED__: app.fingerprint.clone(),
    };

    let launch_info =  __OPENSOURCE__DELETED__LaunchInfo {
        __OPENSOURCE__DELETED__: basic_info,
        __OPENSOURCE__DELETED__: page.clone(),
    };

    let pkt_payload =  __OPENSOURCE__DELETED__ThirdpartyApp {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__thirdparty_app::Payload::LaunchInfo(launch_info)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::LaunchApp as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

fn build_thirdparty_app_uninstall(app: AppInfo) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let basic_info =  __OPENSOURCE__DELETED__BasicInfo {
        __OPENSOURCE__DELETED__: app.package_name.clone(),
        __OPENSOURCE__DELETED__: app.fingerprint.clone(),
    };

    let pkt_payload =  __OPENSOURCE__DELETED__ThirdpartyApp {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__thirdparty_app::Payload::BasicInfo(basic_info)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::RemoveApp as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}


fn build_thirdparty_app_sync_status(app: AppInfo, status: AppStatus) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let basic_info =  __OPENSOURCE__DELETED__BasicInfo {
        __OPENSOURCE__DELETED__: app.package_name.clone(),
        __OPENSOURCE__DELETED__: app.fingerprint.clone(),
    };

    let app_status =  __OPENSOURCE__DELETED__PhoneAppStatus {
        __OPENSOURCE__DELETED__: basic_info,
        __OPENSOURCE__DELETED__: status as i32,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__ThirdpartyApp {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__thirdparty_app::Payload::AppStatus(app_status)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::SyncPhoneAppStatus as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

fn build_thirdparty_app_get_installed() ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::GetInstalledList as u32,
        __OPENSOURCE__DELETED__: None,
    };

    pkt
}

fn build_thirdparty_app_install_request(
    package_name: &String,
    version_code: u32,
    package_size: usize,
) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let install_req =  __OPENSOURCE__DELETED__app_installer::Request {
        __OPENSOURCE__DELETED__: package_name.to_string(),
        __OPENSOURCE__DELETED__: version_code,
        __OPENSOURCE__DELETED__: package_size as u32,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__ThirdpartyApp {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__thirdparty_app::Payload::InstallRequest(install_req)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::ThirdpartyApp as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__thirdparty_app:: __OPENSOURCE__DELETED__::PrepareInstallApp as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::ThirdpartyApp(pkt_payload)),
    };

    pkt
}

use anyhow::{bail, Result};
use prost::Message;
use std::sync::Arc;

use crate::{
    miwear::packet::{Channel, OpCode},
    pb::{self},
};

use super::{
    mass::{packet::MassDataType, SendMassCallbackData},
    MiWearDevice,
};

pub async fn install_firmware<F>(
    device: Arc<MiWearDevice>,
    file_path: &String,
    progress_cb: F,
) -> Result<()>
where
    F: Fn(SendMassCallbackData) + Send + Sync,
{
    let file_data = crate::fs::read_file_cross_platform(file_path).await?;
    let file_md5 = crate::tools::calc_md5(&file_data);

    let firmware_install_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_firmware_install_request(
                "9.9.9".to_string(),
                &file_md5,
                "AstroBox Update".to_string(),
            )
            .encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as u32,
             __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::PrepareOta as u32,
            None,
        )
        .await?;

    match firmware_install_ret.__OPENSOURCE__DELETED__.unwrap() {
         __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::System(system) => match system.__OPENSOURCE__DELETED__.unwrap() {
             __OPENSOURCE__DELETED__system::Payload::PrepareOtaResponse(response) => {
                if response.__OPENSOURCE__DELETED__ ==  __OPENSOURCE__DELETED__PrepareStatus::Ready as i32 {
                    super::mass::send_mass(&device, file_data, MassDataType::FIRMWARE, progress_cb)
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
        },
        _ => {
            bail!("System doesn't exsist!");
        }
    }

    Ok(())
}

fn build_firmware_install_request(
    firmware_version: String,
    file_md5: &Vec<u8>,
    change_log: String,
) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let install_req =  __OPENSOURCE__DELETED__prepare_ota::Request {
        __OPENSOURCE__DELETED__: true,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__prepare_ota::Type::All as i32,
        __OPENSOURCE__DELETED__: firmware_version,
        __OPENSOURCE__DELETED__: crate::tools::to_hex_string(&file_md5),
        __OPENSOURCE__DELETED__: change_log,
        __OPENSOURCE__DELETED__: "".to_owned(),
        __OPENSOURCE__DELETED__: None,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__System {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__system::Payload::PrepareOtaRequest(install_req)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::PrepareOta as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::System(pkt_payload)),
    };

    pkt
}

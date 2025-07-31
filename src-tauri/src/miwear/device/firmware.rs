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
            pb::protocol::wear_packet::Type::System as u32,
            pb::protocol::system::SystemId::PrepareOta as u32,
            None,
        )
        .await?;

    match firmware_install_ret.__OPENSOURCE_DELETED__.unwrap() {
        pb::protocol::wear_packet::Payload::System(system) => match system.__OPENSOURCE_DELETED__.unwrap() {
            pb::protocol::system::Payload::PrepareOtaResponse(response) => {
                if response.__OPENSOURCE_DELETED__ == pb::protocol::PrepareStatus::Ready as i32 {
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
                            "Prepare not READY! Error Info={}",
                            super::error::get_prepare_error_info(response.__OPENSOURCE_DELETED__ as i32)
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
) -> pb::protocol::WearPacket {
    let install_req = pb::protocol::prepare_ota::Request {
        __OPENSOURCE_DELETED__: true,
        __OPENSOURCE_DELETED__: pb::protocol::prepare_ota::Type::All as i32,
        __OPENSOURCE_DELETED__: firmware_version,
        __OPENSOURCE_DELETED__: crate::tools::to_hex_string(&file_md5),
        __OPENSOURCE_DELETED__: change_log,
        __OPENSOURCE_DELETED__: "".to_owned(),
        __OPENSOURCE_DELETED__: None,
    };

    let pkt_payload = pb::protocol::System {
        __OPENSOURCE_DELETED__: Some(pb::protocol::system::Payload::PrepareOtaRequest(install_req)),
    };

    let pkt = pb::protocol::WearPacket {
        __OPENSOURCE_DELETED__: pb::protocol::wear_packet::Type::System as i32,
        __OPENSOURCE_DELETED__: pb::protocol::system::SystemId::PrepareOta as u32,
        __OPENSOURCE_DELETED__: Some(pb::protocol::wear_packet::Payload::System(pkt_payload)),
    };

    pkt
}

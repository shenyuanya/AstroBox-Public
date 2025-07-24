use crate::miwear::MiWearDevice;
use crate::{
    miwear::packet::{Channel, OpCode},
    pb::{self},
};
use anyhow::{bail, Result};
use prost::Message;
use serde::Serialize;
use std::{sync::Arc};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq,Serialize)]
pub enum ChargeStatus {
    Unknown = 0,
    Charging = 1,
    NotCharging = 2,
    Full = 3,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemStatus {
    pub capacity: u32,
    pub charge_status: ChargeStatus,
    pub charge_info: Option<ChargeInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChargeInfo {
    pub state: u32,
    pub timestamp: Option<u32>,
}

#[derive(Debug, Clone,Serialize)]
pub struct SystemInfo {
    pub serial_number: String,
    pub firmware_version: String,
    pub imei: String,
    pub model: String,
}

pub async fn system_get_device_status(
    device: Arc<MiWearDevice>,
) -> Result<SystemStatus> {

    let device_status_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_system_get_device_status()
                .encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as u32,
             __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::GetDeviceStatus as u32,
            None,
        )
        .await?;

    if let Some(payload) = device_status_ret.__OPENSOURCE__DELETED__ {
        match payload {
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::System(system) => {
                if let Some(system_payload) = system.__OPENSOURCE__DELETED__ {
                    match system_payload {
                         __OPENSOURCE__DELETED__system::Payload::DeviceStatus(status) => {

                            let mut result_status = ChargeStatus::Unknown;

                            if let Some(charge_status) = status.battery.charge_status {
                                if charge_status == 1 {
                                    result_status = ChargeStatus::Charging;
                                }
                                else if charge_status == 2 {
                                    result_status = ChargeStatus::NotCharging;
                                }
                                else if charge_status == 3 {
                                    result_status = ChargeStatus::Full;
                                }
                            }
                            let charge_info = match status.battery.charge_info {
                                Some(charge_info) => {
                                    Some(ChargeInfo {
                                        state: charge_info.state,
                                        timestamp: charge_info.timestamp,
                                    })
                                }
                                None => None,
                            };

                            let result = SystemStatus {
                                capacity: status.battery.capacity,
                                charge_status: result_status,
                                charge_info: charge_info,
                            };

                            return Ok(result);
                        },
                        _ => {
                            bail!("Status doesn't exsist!");
                        },
                    }
                }
            },
            _ => {
                bail!("System doesn't exsist!");
            },
        }
    }

    bail!("Packet doesn't exsist!");
}

pub async fn system_get_device_info(
    device: Arc<MiWearDevice>,
) -> Result<SystemInfo> {

        let device_info_ret = device
        .request_proto(
            Channel::Pb,
            OpCode::Encrypted,
            &build_system_get_device_info()
                .encode_to_vec(),
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as u32,
             __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::GetDeviceInfo as u32,
            None,
        )
        .await?;

    if let Some(payload) = device_info_ret.__OPENSOURCE__DELETED__ {
        match payload {
             __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::System(system) => {
                if let Some(system_payload) = system.__OPENSOURCE__DELETED__ {
                    match system_payload {
                         __OPENSOURCE__DELETED__system::Payload::DeviceInfo(info) => {

                            let result = SystemInfo {
                                serial_number: info.__OPENSOURCE__DELETED__.clone(),
                                firmware_version: info.__OPENSOURCE__DELETED__.clone(),
                                imei: info.__OPENSOURCE__DELETED__.clone(),
                                model: info.__OPENSOURCE__DELETED__.clone(),
                            };

                            return Ok(result);
                        },
                        _ => {
                            bail!("Info doesn't exsist!");
                        },
                    }
                }
            },
            _ => {
                bail!("System doesn't exsist!");
            },
        }
    }

    bail!("Packet doesn't exsist!");
}

fn build_system_get_device_status() ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::GetDeviceStatus as u32,
        __OPENSOURCE__DELETED__: None,
    };

    pkt

}

fn build_system_get_device_info() ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::System as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__system:: __OPENSOURCE__DELETED__::GetDeviceInfo as u32,
        __OPENSOURCE__DELETED__: None,
    };

    pkt

}
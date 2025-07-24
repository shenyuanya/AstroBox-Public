use std::sync::Arc;

use super::MiWearDevice;
use crate::{
    crypto::aesccm::aes128_ccm_encrypt,
    miwear::packet::{Channel, OpCode},
    pb::{self},
};
use anyhow::anyhow;
use anyhow::{bail, Result};
use hmac::{Hmac, Mac};
use prost::Message;
use sha2::Sha256;

pub async fn do_auth(device: Arc<MiWearDevice>) -> Result<()> {
    let nonce = crate::tools::generate_random_bytes(16);

    let app_verify = build_auth_step_1(&nonce);
    let ret = device
        .request(
            Channel::Pb,
            OpCode::Plain,
            &app_verify.encode_to_vec(),
            None,
        )
        .await?;

    log::info!("[MiWearDevice.Auth] do_auth get app_verify ret.");

    match ret {
        crate::miwear::packet::PacketData::PROTOBUF( __OPENSOURCE__DELETED__) => {
            if  __OPENSOURCE__DELETED__.__OPENSOURCE__DELETED__ ==  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::Account as i32 {
                match  __OPENSOURCE__DELETED__.__OPENSOURCE__DELETED__.unwrap() {
                     __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Account(account) => match account.__OPENSOURCE__DELETED__.unwrap() {
                         __OPENSOURCE__DELETED__account::Payload::AuthDeviceVerify(auth_device_verify) => {
                            let app_confirm =
                                build_auth_step_2(&device, &nonce, &auth_device_verify).await?;
                            let ret = device
                                .request(
                                    Channel::Pb,
                                    OpCode::Plain,
                                    &app_confirm.encode_to_vec(),
                                    None,
                                )
                                .await?;

                            match ret {
                                crate::miwear::packet::PacketData::PROTOBUF( __OPENSOURCE__DELETED__) => {
                                    match  __OPENSOURCE__DELETED__.__OPENSOURCE__DELETED__.unwrap() {
                                         __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Account(account) => {
                                            match account.__OPENSOURCE__DELETED__.unwrap() {
                                                 __OPENSOURCE__DELETED__account::Payload::AuthDeviceConfirm(
                                                    auth_device_confirm,
                                                ) => {
                                                    log::info!(
                                                        "[MiWearDevice.Auth] Auth Finished: {}",
                                                        auth_device_confirm.__OPENSOURCE__DELETED__
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {
                                    bail!("Packet type error");
                                }
                            }
                        }
                        _ => {
                            bail!("Auth device verify field was not present in the received protobuf.");
                        }
                    },
                    _ => {
                        bail!("Account field was not present in the received protobuf.");
                    }
                }
            }
        }
        _ => {
            bail!("Packet type error");
        }
    }

    log::info!("[MiWearDevice.Auth] Auth Ok");

    Ok(())
}

fn build_auth_step_1(nonce: &[u8]) ->  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
    let account_payload =  __OPENSOURCE__DELETED__auth::AppVerify {
        __OPENSOURCE__DELETED__: nonce.to_vec(),
        __OPENSOURCE__DELETED__: None,
        __OPENSOURCE__DELETED__: None,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__Account {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__account::Payload::AuthAppVerify(account_payload)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::Account as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__account::AccountId::AuthVerify as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Account(pkt_payload)),
    };

    pkt
}

async fn build_auth_step_2(
    device: &Arc<MiWearDevice>,
    p_random: &[u8],
    device_verify: & __OPENSOURCE__DELETED__auth::DeviceVerify,
) -> Result< __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__> {
    let mut state = device.state.write().await;

    let w_random = device_verify.__OPENSOURCE__DELETED__.clone();
    let w_sign = device_verify.__OPENSOURCE__DELETED__.clone();

    if w_random.len() != 16 || w_sign.len() != 32 {
        return Err(anyhow!("nonce/hmac length mismatch"));
    }

    let block = kdf_miwear(
        &string_to_u8_16(&state.authkey).unwrap(),
        p_random.try_into().unwrap(),
        &w_random.clone().try_into().unwrap(),
    );
    let dec_key = &block[0..16];
    let enc_key = &block[16..32];
    let dec_nonce = &block[32..36];
    let enc_nonce = &block[36..40];

    // 校验 HMAC ---
    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(dec_key).unwrap();
    mac.update(&w_random);
    mac.update(p_random);
    let expect = mac.finalize().into_bytes();
    if w_sign != expect.as_slice() {
        return Err(anyhow!("Auth HMAC mismatch - wrong auth key?"));
    }

    // encryptedSigns (HMAC) ---
    let mut mac2 = hmac::Hmac::<Sha256>::new_from_slice(enc_key).unwrap();
    mac2.update(p_random);
    mac2.update(&w_random);
    let encrypted_signs = mac2.finalize().into_bytes().to_vec(); // 32 B

    let proto_companion_device =  __OPENSOURCE__DELETED__CompanionDevice {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__companion_device::DeviceType::Android as i32, // 写死安卓以解锁全部功能
        __OPENSOURCE__DELETED__: None,
        __OPENSOURCE__DELETED__: "AstroBox".to_string(),
        __OPENSOURCE__DELETED__: Some(0xffffffff), // 解锁全部capability 0xffffffff
        __OPENSOURCE__DELETED__: None,
        __OPENSOURCE__DELETED__: None,
    };

    let companion_device = proto_companion_device.encode_to_vec();
    let pkt_nonce = {
        let mut n = Vec::with_capacity(12);
        n.extend_from_slice(enc_nonce); // 4B
        n.extend_from_slice(&0u32.to_le_bytes()); // counterHi
        n.extend_from_slice(&0u32.to_le_bytes()); // counterLo (i=0)
        n
    };
    let encrypted_device_info = aes128_ccm_encrypt(
        enc_key.try_into().unwrap(),
        &pkt_nonce.try_into().unwrap(),
        &[],
        &companion_device,
    );

    let account_payload =  __OPENSOURCE__DELETED__auth::AppConfirm {
        __OPENSOURCE__DELETED__: encrypted_signs,
        __OPENSOURCE__DELETED__: encrypted_device_info,
    };

    let pkt_payload =  __OPENSOURCE__DELETED__Account {
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__account::Payload::AuthAppConfirm(account_payload)),
    };

    let pkt =  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__ {
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Type::Account as i32,
        __OPENSOURCE__DELETED__:  __OPENSOURCE__DELETED__account::AccountId::AuthConfirm as u32,
        __OPENSOURCE__DELETED__: Some( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__::Payload::Account(pkt_payload)),
    };

    state.sec_keys = Some(super::SecurityKeys {
        enc_key: enc_key.to_vec(),
        dec_key: dec_key.to_vec(),
        enc_nonce: enc_nonce.to_vec(),
        dec_nonce: dec_nonce.to_vec(),
    });

    Ok(pkt)
}

fn string_to_u8_16(s: &String) -> Option<[u8; 16]> {
    if s.len() != 32 {
        return None;
    }

    let mut result = [0u8; 16];
    for i in 0..16 {
        let byte_str = &s[i * 2..i * 2 + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(val) => result[i] = val,
            Err(_) => return None,
        }
    }
    Some(result)
}

fn kdf_miwear(secret_key: &[u8; 16], phone_nonce: &[u8; 16], watch_nonce: &[u8; 16]) -> [u8; 64] {
    type HmacSha256 = Hmac<Sha256>;

    // 1) hmac_key = HMAC(init_key, secret_key)
    let mut init_key = [0u8; 32];
    init_key[..16].copy_from_slice(phone_nonce);
    init_key[16..].copy_from_slice(watch_nonce);

    let mut mac = HmacSha256::new_from_slice(&init_key).expect("HMAC key length fixed");
    mac.update(secret_key);
    let hmac_key = mac.finalize().into_bytes(); // 32 B

    // 2) expand to 64 B
    let mut okm = [0u8; 64];
    let tag = b"miwear-auth";
    let mut offset = 0;
    let mut prev: Vec<u8> = Vec::new();
    for counter in 1u8..=3 {
        // 3*32 >= 64
        let mut mac = HmacSha256::new_from_slice(&hmac_key).unwrap();
        mac.update(&prev);
        mac.update(tag);
        mac.update(&[counter]);
        prev = mac.finalize().into_bytes().to_vec();
        let end = (offset + 32).min(64);
        okm[offset..end].copy_from_slice(&prev[..end - offset]);
        offset = end;
    }
    okm
}

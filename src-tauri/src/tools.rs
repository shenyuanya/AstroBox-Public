use anyhow::Context;
use crc::{Crc, CRC_32_ISO_HDLC};
use md5::{Digest, Md5};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Sha256};
use tauri_plugin_opener::OpenerExt;

pub fn to_hex_string(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_stream_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string has an odd length".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

pub fn vec_to_array_16_opt(v: &Vec<u8>) -> Option<[u8; 16]> {
    v.as_slice().try_into().ok()
}

/// 判断 UUID 中是否包含指定目标字符串
///
/// - `uuid`: 要检查的 UUID
/// - `target`: 要查找的子串（如 "fe95"）
/// - `only_prefix`: 如果为 `true`，只检查第一个 '-' 前的部分；否则在整个 UUID 字符串中查找
pub fn uuid_contains(uuid: &uuid::Uuid, target: &str, only_prefix: bool) -> bool {
    let uuid_str = uuid.to_string();

    if only_prefix {
        // 查找第一个 '-' 并检查其前缀
        if let Some(idx) = uuid_str.find('-') {
            let prefix = &uuid_str[..idx];
            prefix.contains(target)
        } else {
            false
        }
    } else {
        uuid_str.contains(target)
    }
}

pub fn generate_random_bytes(size: usize) -> Vec<u8> {
    // Corrected: Use rand::thread_rng() for rand 0.8.5
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; size];
    rng.fill(&mut buffer[..]);
    buffer
}

pub fn random_string(n: usize) -> String {
    // Corrected: Use rand::thread_rng() for rand 0.8.5
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

pub fn calc_md5(data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);

    hasher.finalize().to_vec()
}

pub fn calc_crc32_bytes(data: &[u8]) -> Vec<u8> {
    const STANDARD_CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let crc_value = STANDARD_CRC32.checksum(data);
    crc_value.to_be_bytes().to_vec()
}

/// 使用系统默认浏览器打开指定 URL
pub fn open_url_with_default_browser(url: String) -> anyhow::Result<()> {
    let handle = crate::APP_HANDLE
        .get()
        .context("APP_HANDLE 尚未初始化")?;
    handle
        .opener()
        .open_url(&url, None::<&str>)
        .context("打开链接失败")?;
    Ok(())
}

pub fn calc_unlock_code(mac: String, sn: String) -> String {

    let mac = mac.replace(":", "").trim().to_string();
    let sn = sn.trim().to_string();

    let mut hasher = Sha256::default();
    hasher.update(mac);
    hasher.update(sn);
    hasher.update("XIAOMI");

    let hash = hasher.finalize();

    let mut code = String::new();
    for i in 0..10 {
        let k = hash[i as usize] % 0xA;
        code.push_str(&k.to_string());
    }

    return code;
}
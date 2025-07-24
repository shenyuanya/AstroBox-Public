use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use cipher::StreamCipher;
use md5::Md5;
use rand::{distributions::Uniform, rngs::OsRng, Rng};
use rc4::{consts::U32, KeyInit, Rc4};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex, RawCookie};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceLoginAuthRespone {
    #[serde(default)]
    pub qs: String,
    #[serde(default)]
    pub ssecurity: String,
    pub code: u32,
    #[serde(rename = "passToken")]
    #[serde(default)]
    pub pass_token: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "securityStatus")]
    pub security_status: u32,
    #[serde(default)]
    pub nonce: u64,
    #[serde(rename = "userId")]
    #[serde(default)]
    pub user_id: u32,
    #[serde(rename = "cUserId")]
    #[serde(default)]
    pub c_user_id: String,
    pub result: String,
    #[serde(default)]
    pub psecurity: String,
    pub location: String,
    pub pwd: u32,
    pub child: u32,
    #[serde(default)]
    pub desc: String,
    #[serde(rename = "notificationUrl")]
    pub notification_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceLoginRespone {
    #[serde(rename = "_sign")]
    pub sign: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceDetail {
    #[serde(default)]
    pub beaconkey: String,
    #[serde(default)]
    pub encrypt_key: String,
    #[serde(default)]
    pub fw_ver: String,
    #[serde(default)]
    pub irq_key: String,
    #[serde(default)]
    pub last_bind_time: String,
    #[serde(default)]
    pub mac: String,
    #[serde(default)]
    pub phone_id: String,
    #[serde(default)]
    pub sn: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceInfo {
    pub sid: String,
    pub identifier: String,
    pub name: String,
    pub model: String,
    pub status: u32,
    pub create_time: u64,
    pub update_time: u64,
    pub detail: DeviceDetail,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceListResult {
    pub list: Vec<DeviceInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceListRespone {
    pub code: u32,
    pub message: String,
    pub result: DeviceListResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiAccountToken {
    pub ssecurity: String,
    pub service_token: String,
    pub c_user_id: String,
}

/// Remove Xiaomi’s magic prefix from JSON payloads.
fn strip_prefix(contents: &str) -> &str {
    const PREFIX: &str = "&&&START&&&";
    contents.strip_prefix(PREFIX).unwrap_or(contents)
}

/// Generate Mi nonce: 8 random bytes + 4-byte minutes since epoch (big-endian).
fn generate_nonce(millis: u64) -> String {
    let mut rand_part = [0u8; 8];
    OsRng.fill(&mut rand_part);
    let mut buf = Vec::with_capacity(12);
    buf.extend_from_slice(&rand_part);
    buf.extend_from_slice(&((millis / 60_000) as u32).to_be_bytes());
    general_purpose::STANDARD.encode(buf)
}

/// 6-character lowercase device id.
fn random_device_id() -> String {
    let range = Uniform::from(b'a'..=b'z');
    OsRng
        .sample_iter(&range)
        .take(6)
        .map(char::from)
        .collect()
}

/// SHA-256( ssecurity_b64_dec + nonce_b64_dec ) → Base64.
fn calc_signed_nonce(ssecurity: &str, nonce: &str) -> Result<String> {
    let mut hasher = Sha256::default();
    hasher.update(&general_purpose::STANDARD.decode(ssecurity)?);
    hasher.update(&general_purpose::STANDARD.decode(nonce)?);
    Ok(general_purpose::STANDARD.encode(hasher.finalize()))
}

/// Deterministic Xiaomi signature builder (ASCII sort on keys).
fn generate_enc_signature(
    url_path: &str,
    method: &str,
    signed_nonce: &str,
    params: &HashMap<String, String>,
) -> String {
    let mut keys: Vec<&String> = params.keys().collect();
    keys.sort(); // ASCII order
    let mut pieces = Vec::with_capacity(2 + keys.len() + 1);
    pieces.push(method.to_uppercase());
    pieces.push(url_path.to_owned());
    for k in keys {
        pieces.push(format!("{k}={}", params.get(k).unwrap()));
    }
    pieces.push(signed_nonce.to_owned());

    let raw = pieces.join("&");
    let mut sha1 = Sha1::default();
    sha1.update(raw.as_bytes());
    general_purpose::STANDARD.encode(sha1.finalize())
}

/// Encrypt payload parameters with RC4, respecting deterministic key order.
fn rc4_encrypt_params(
    signed_nonce: &str,
    params_plain: &HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    // Build a cipher, drop first 1024 bytes.
    let key_bytes = general_purpose::STANDARD.decode(signed_nonce)?;
    let key = rc4::Key::<U32>::from_slice(&key_bytes);
    let mut cipher = Rc4::<U32>::new(key);
    let mut drop_buf = [0u8; 1024];
    cipher.apply_keystream(&mut drop_buf);

    // Encrypt in deterministic order.
    let mut keys: Vec<&String> = params_plain.keys().collect();
    keys.sort(); // ASCII order

    let mut encrypted = HashMap::new();
    for k in keys {
        let mut data = params_plain.get(k).unwrap().as_bytes().to_vec();
        cipher.apply_keystream(&mut data);
        encrypted.insert(k.to_string(), general_purpose::STANDARD.encode(data));
    }
    Ok(encrypted)
}

/// Main encrypted API call.
/// `prefix` – path prefix that should be trimmed before signing (usually "")
pub async fn mi_service_call_encrypted(
    token: MiAccountToken,
    prefix: String,
    url: String,
    mut params_plain: HashMap<String, String>,
    ua: String,
) -> Result<String> {
    let client = crate::net::default_client_builder()
        .build()?;

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;
    let nonce = generate_nonce(millis);
    let signed_nonce = calc_signed_nonce(&token.ssecurity, &nonce)?;

    // Path for signature
    let url_parsed = Url::parse(&url)?;
    let url_path = url_parsed
        .path()
        .trim_start_matches(&prefix)
        .to_string();

    // 1. rc4_hash__ over *plain* params
    let rc4_hash = generate_enc_signature(&url_path, "POST", &signed_nonce, &params_plain);
    params_plain.insert("rc4_hash__".to_string(), rc4_hash);

    // 2. RC4-encrypt all fields
    let mut params_enc = rc4_encrypt_params(&signed_nonce, &params_plain)?;

    // 3. Final signature & nonce
    let sig = generate_enc_signature(&url_path, "POST", &signed_nonce, &params_enc);
    params_enc.insert("signature".into(), sig);
    params_enc.insert("_nonce".into(), nonce.clone());

    // 4. Headers & cookies
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_str(&ua)?);
    headers.insert("region_tag", HeaderValue::from_static("cn"));
    headers.insert("HandleParams", HeaderValue::from_static("true"));

    let cookie_header = format!(
        "cUserId={}; serviceToken={}; locale=en_us;",
        token.c_user_id, token.service_token
    );

    // 5. Request
    let resp = client
        .post(url)
        .headers(headers)
        .header("Cookie", cookie_header)
        .form(&params_enc)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        return Err(anyhow!("Mi API call failed: {}, body: {}", status, body));
    }

    // 6. Decrypt
    let key_bytes = general_purpose::STANDARD.decode(&signed_nonce)?;
    let key = rc4::Key::<U32>::from_slice(&key_bytes);
    let mut cipher = Rc4::<U32>::new(key);
    let mut drop_buf = [0u8; 1024];
    cipher.apply_keystream(&mut drop_buf);

    let mut data = general_purpose::STANDARD.decode(body.trim_matches('"'))?;
    cipher.apply_keystream(&mut data);
    Ok(String::from_utf8_lossy(&data).into_owned())
}

/// Log in and obtain `MiAccountToken`.
pub async fn login_mi_account(username: String, password: String, ua: String) -> Result<MiAccountToken> {
    let device_id = random_device_id();
    let sdk_version = "accountsdk-18.8.15";

    let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
    let client = crate::net::default_client_builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    // Pre-seed cookies on mi.com + xiaomi.com + account.xiaomi.com
    for domain in ["https://mi.com", "https://xiaomi.com", "https://account.xiaomi.com"] {
        let url = Url::parse(domain)?;
        let mut store = cookie_store.lock().unwrap();
        store.insert_raw(&RawCookie::new("sdkVersion", sdk_version), &url)?;
        store.insert_raw(&RawCookie::new("deviceId", device_id.clone()), &url)?;
        if domain.contains("xiaomi.com") {
            // Ensure userId is available to account.xiaomi.com just like Python implementation.
            let raw = RawCookie::parse(format!(
                "userId={}; Domain=.xiaomi.com; Path=/",
                username
            ))?;
            store.insert_raw(&raw, &url)?;
        }
    }

    // --- Step 1: get _sign ---
    let step1 = client
        .get("https://account.xiaomi.com/pass/serviceLogin?sid=miothealth&_json=true")
        .header("User-Agent", ua.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;

    if step1.status() != 200 {
        return Err(anyhow!("serviceLogin failed: {}", step1.status()));
    }
    let sign_resp: ServiceLoginRespone =
        serde_json::from_str(strip_prefix(&step1.text().await?))?;
    let sign = sign_resp.sign;

    if sign.is_empty() {
        return Err(anyhow!("_sign not found in serviceLogin response"));
    }

    // --- Step 2: serviceLoginAuth2 ---
    let mut md5 = Md5::default();
    md5.update(password.as_bytes());
    let pwd_hash = format!("{:x}", md5.finalize()).to_uppercase();

    let mut fields = HashMap::<&str, &str>::new();
    fields.insert("sid", "miothealth");
    fields.insert("hash", &pwd_hash);
    fields.insert("callback", "https://sts-hlth.io.mi.com/healthapp/sts");
    fields.insert("qs", "%3Fsid%3Dmiothealth%26_json%3Dtrue");
    fields.insert("user", &username);
    fields.insert("_sign", &sign);
    fields.insert("_json", "true");

    log::info!("1");

    let step2 = client
        .post("https://account.xiaomi.com/pass/serviceLoginAuth2")
        .header("User-Agent", ua.to_owned())
        .form(&fields)
        .send()
        .await?;

    if step2.status() != 200 {
        return Err(anyhow!("serviceLoginAuth2 failed: {}", step2.status()));
    }

    let auth_resp: ServiceLoginAuthRespone =
        serde_json::from_str(strip_prefix(&step2.text().await?))?;

    log::info!("2");

    if auth_resp.code != 0 || auth_resp.ssecurity.is_empty() {
        match auth_resp.code {
            0 => {}
            70016 => {
                return Err(anyhow!("登录失败：用户名或密码错误"))
            }
            _ => {
                return Err(anyhow!(
                    "未知错误：serviceLoginAuth error: code={}, description={}",
                    auth_resp.code,
                    auth_resp.description
                ));
            }
        }
    }

    if let Some(url) = &auth_resp.notification_url {
        return Err(anyhow!(
            "2-f-a={}",
            url
        ))
    }

    // --- Step 3: follow redirect to obtain serviceToken ---
    let step3 = client
        .get(&auth_resp.location)
        .header("User-Agent", ua)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;

    if step3.status() != 200 {
        return Err(anyhow!("location redirect failed: {}", step3.status()));
    }

    let store = cookie_store.lock().unwrap();
    let service_token = store
        .get("sts-hlth.io.mi.com", "/", "serviceToken")
        .map(|ck| ck.value().to_string())
        .ok_or_else(|| anyhow!("serviceToken cookie missing"))?;

    log::info!("3");

    Ok(MiAccountToken {
        ssecurity: auth_resp.ssecurity,
        service_token,
        c_user_id: auth_resp.c_user_id,
    })
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SPPDevice {
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectArg {
    pub addr: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectResult {
    pub ret: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScannedDevicesResult {
    pub ret: Vec<SPPDevice>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDataListenerResult {
    pub ret: String,
    pub err: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SPPSendPayload {
    pub b64data: String,
}

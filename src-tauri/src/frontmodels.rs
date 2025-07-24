use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BTDeviceInfo {
    pub name: String,
    pub addr: String,
    pub connect_type: String,
}

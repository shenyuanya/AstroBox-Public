use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type DeviceMap = HashMap<String, DeviceMapItem>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMapItem {
    pub name: String,
    pub codename: String,
    pub chip: String,
    pub fetch: bool
}
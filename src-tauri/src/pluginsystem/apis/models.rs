use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    pub method: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub raw: bool,
    pub body_encoded : bool,// Body在astrobox-plugin-sdk如过是u8array中被转换成base64,这项为true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeviceListReturn {
    pub name: String,
    pub addr: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickFileReturn {
    pub path: String,
    pub size: u64,
    pub text_len: u64
}
#[derive(Debug, Clone,Default, Serialize, Deserialize)]
pub struct PickFileOptions {
    pub decode_text: bool,
    pub encoding: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileOptions {
    pub offset: u64,
    pub len: u64,
    #[serde(default)]
    pub decode_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PluginUINodeContent {
    Text(String),
    Button(PluginUIButton),
    Dropdown(PluginUIDropdown),
    Input(PluginUIInput),
    HtmlDocument(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUIButton {
    pub primary: bool,
    pub text: String,
    pub callback_fun_id: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUIDropdown {
    pub options: Vec<String>,
    pub callback_fun_id: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUIInput {
    pub text: String,
    pub callback_fun_id: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUINode {
    pub node_id: String,
    pub visibility: bool,
    pub disabled: bool,
    pub content: PluginUINodeContent
}

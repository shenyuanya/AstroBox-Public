use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String, // 插件名称
    pub icon: String, // 插件图标（路径）
    pub version: String, // 插件版本
    pub description: String, // 插件简介
    pub author: String, // 插件作者
    pub website: String, // 插件网站（例如github仓库地址）
    pub entry: String, // 插件入口js文件
    pub api_level: u32, // 插件api等级
    pub permissions: Vec<String>, // 插件权限列表
    #[serde(default)]
    pub additional_files: Vec<String> // 插件附加文件列表
}
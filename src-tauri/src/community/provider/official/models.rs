use serde::{Deserialize, Deserializer, Serialize};

fn split_semicolon<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.split(';').map(|x| x.trim().to_string()).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRes {
    pub name: String,
    pub icon: String,
    pub cover: String,
    pub restype: String,
    #[serde(deserialize_with = "split_semicolon")]
    pub tags: Vec<String>,
    #[serde(deserialize_with = "split_semicolon")]
    pub devices: Vec<String>,
    pub path: String,
    #[serde(default)]
    pub paid_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDest {
    pub manifest_ver: u32,
    pub repo_url: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Banner {
    pub background: String,
    pub title: String,
    pub description: String,
    pub foreground: String,
    pub button: BannerButton
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerButton {
    pub text: String,
    pub url: String
}
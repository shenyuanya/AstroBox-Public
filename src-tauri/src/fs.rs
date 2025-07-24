use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::fs::File;
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

#[cfg(target_os = "android")]
use url::Url;

fn to_filepath(path: &str) -> Result<FilePath> {
    #[cfg(target_os = "android")]
    {
        // `content://…` / `file://…` / 其它 URI
        if path.contains("://") {
            return Ok(FilePath::from(Url::parse(path)?));
        }
    }
    // 其它平台或普通本地路径
    Ok(FilePath::from(PathBuf::from(path)))
}

pub async fn read_file_cross_platform(path: impl AsRef<str>) -> Result<Vec<u8>> {
    let app = crate::APP_HANDLE
        .get()
        .ok_or_else(|| anyhow!("APP_HANDLE 未初始化"))?;

    let file_path = to_filepath(path.as_ref())?;
    Ok(app.fs().read(file_path)?)
}

pub async fn open_file_cross_platform(path: impl AsRef<str>) -> Result<File> {
    let app = crate::APP_HANDLE
        .get()
        .ok_or_else(|| anyhow!("APP_HANDLE 未初始化"))?;

    let file_path = to_filepath(path.as_ref())?;

    let mut opts = OpenOptions::new();
    opts.read(true);

    Ok(app.fs().open(file_path, opts)?)
}

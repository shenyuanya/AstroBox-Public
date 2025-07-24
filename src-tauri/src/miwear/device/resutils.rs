use std::path::PathBuf;
use url::Url;

pub async fn get_file_type(path: &str) -> Result<String, std::io::Error> {
    let data = crate::fs::read_file_cross_platform(&path.to_string()).await.unwrap_or(vec![]);

    if data.is_empty() {
        return Ok("null".to_string());
    }

    // 提取文件名，兼容 URL 和路径
    fn extract_filename(s: &str) -> Option<PathBuf> {
        // 如果是URL
        if let Ok(url) = Url::parse(s) {
            // path_segments 最后一个就是文件名
            if let Some(segments) = url.path_segments() {
                if let Some(filename) = segments.last() {
                    return Some(PathBuf::from(filename));
                }
            }
            return None;
        }
        // 否则按本地路径
        Some(PathBuf::from(s).file_name().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(s)))
    }

    let filename = extract_filename(path).unwrap_or_else(|| PathBuf::from(path));

    // 1. 检查是不是 ZIP 格式
    if data.len() >= 4 && &data[..4] == [0x50, 0x4B, 0x03, 0x04] {
        // 检查扩展名 abp
        if let Some(ext) = filename.extension() {
            if ext == "abp" {
                return Ok("abp".to_string());
            }
        }
        // 检查尾部是否包含 quickapp 字样
        let tail = if data.len() > 256 {
            &data[data.len() - 256..]
        } else {
            &data[..]
        };
        if String::from_utf8_lossy(tail).contains("toolkit") {
            return Ok("quickapp".to_string());
        } else {
            return Ok("zip".to_string());
        }
    }

    // 2. 检查是不是文本（utf8）
    if std::str::from_utf8(&data).is_ok() {
        return Ok("text".to_string());
    }

    // 3. 检查小米表盘魔数 5a a5 34 12
    if data.len() >= 4 && &data[..4] == [0x5a, 0xa5, 0x34, 0x12] {
        return Ok("watchface".to_string());
    }

    // 4. 其它都认为是二进制
    Ok("binary".to_string())
}

pub async fn get_watchface_id(path: &str) -> Option<String> {
    let data = crate::fs::read_file_cross_platform(&path.to_string())
        .await
        .unwrap_or(vec![]);
    if data.len() < 34 + 16 {
        return None;
    }
    let id_bytes = &data[34..34 + 16];
    let watchface_id = String::from_utf8_lossy(id_bytes).to_string();
    log::info!("watchface_id: {}", watchface_id);
    Some(watchface_id)
}

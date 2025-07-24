use base64::{engine::general_purpose, Engine};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsString, JsValue, NativeFunction};
use chardetng::EncodingDetector;
use dashmap::{DashMap, DashSet};
use encoding_rs::{BIG5, EUC_JP, EUC_KR, GBK, ISO_8859_2, SHIFT_JIS, UTF_8, WINDOWS_1252};
use once_cell::sync::Lazy;
use std::{
    io::{Read, Seek, SeekFrom},
    str::FromStr,
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath;
use tokio::sync::oneshot;

use crate::pluginsystem::apis::models::PickFileOptions;
use crate::pluginsystem::{
    apis::models::{PickFileReturn, ReadFileOptions},
    utils::plugin_permission_check,
};

static ALLOWED_PATHS: Lazy<DashSet<String>> = Lazy::new(DashSet::new);

pub static FILESYSTEM_PERMISSION: &str = "filesystem";

static READ_FILES: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

fn normalize_path(path: &str) -> String {
    if path.contains("://") {
        return path.to_owned();
    }
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_owned())
}

async fn ensure_allowed(path: &str) -> JsResult<()> {
    if ALLOWED_PATHS.contains(path) {
        return Ok(());
    }

    if !path.contains("://") {
        let canon = tokio::task::spawn_blocking({
            let p = path.to_owned();
            move || std::fs::canonicalize(p)
        })
        .await
        .map_err(|_| js_error!("canonicalize join error"))?
        .map_err(|_| js_error!("canonicalize failed"))?;
        let canon_str = canon.to_string_lossy();
        if ALLOWED_PATHS.contains(canon_str.as_ref()) {
            return Ok(());
        }
    }

    Err(js_error!("Path not allowed, pick it first"))
}

pub fn pick_file_dialog(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> impl std::future::Future<Output = JsResult<JsValue>> {
    let permission_result = plugin_permission_check(ctx, FILESYSTEM_PERMISSION.to_string());

    let options_obj:Option<PickFileOptions> = args.get(0)
        .and_then(|v| v.to_string(ctx).ok())
        .and_then(|s| serde_json::from_str(&s.to_std_string_lossy()).ok());

    let PickFileOptions{decode_text, encoding} = options_obj.unwrap_or_default();
    async move {
        if let Some(err) = permission_result {
            return Err(err);
        }

        let (tx, rx) = oneshot::channel::<Option<FilePath>>();

        if let Some(app) = crate::APP_HANDLE.get() {
            app.dialog().file().pick_file(move |file_path| {
                let _ = tx.send(file_path);
            });
        } else {
            return Err(js_error!("APP_HANDLE not initialized"));
        }

        match rx.await {
            Ok(Some(file_path)) => {
                let path_str = file_path.to_string();
                let norm = normalize_path(&path_str);

                ALLOWED_PATHS.insert(path_str.clone());
                ALLOWED_PATHS.insert(norm.clone());

                let f = crate::fs::open_file_cross_platform(&path_str)
                    .await
                    .map_err(|_| js_error!("open file failed"))?;

                let metadata = f.metadata().map_err(|_| js_error!("open file failed"))?;
                let size = if decode_text {
                    let mut buf = Vec::new();
                    f.take(u64::MAX).read_to_end(&mut buf).map_err(|_| js_error!("read file failed"))?;

                    let text = if let Some(enc_name) = encoding {
                        match enc_name.to_lowercase().as_str() {
                            "utf-8" | "utf8" => String::from_utf8_lossy(&buf).into_owned(),
                            "gbk" => {
                                let (cow, _, _) = GBK.decode(&buf);
                                cow.into_owned()
                            }
                            "big5" => {
                                let (cow, _, _) = BIG5.decode(&buf);
                                cow.into_owned()
                            }
                            "shift_jis" | "sjis" => {
                                let (cow, _, _) = SHIFT_JIS.decode(&buf);
                                cow.into_owned()
                            }
                            "euc-jp" => {
                                let (cow, _, _) = EUC_JP.decode(&buf);
                                cow.into_owned()
                            }
                            "euc-kr" => {
                                let (cow, _, _) = EUC_KR.decode(&buf);
                                cow.into_owned()
                            }
                            "windows-1252" => {
                                let (cow, _, _) = WINDOWS_1252.decode(&buf);
                                cow.into_owned()
                            }
                            "iso-8859-2" => {
                                let (cow, _, _) = ISO_8859_2.decode(&buf);
                                cow.into_owned()
                            }
                            _ => String::from_utf8_lossy(&buf).into_owned(),
                        }
                    } else {
                        let mut detector = EncodingDetector::new();
                        detector.feed(&buf, true);
                        let encoding = detector.guess(None, true);

                        let mut candidates = vec![
                            encoding,
                            UTF_8,
                            GBK,
                            BIG5,
                            SHIFT_JIS,
                            EUC_JP,
                            EUC_KR,
                            WINDOWS_1252,
                            ISO_8859_2,
                        ];
                        candidates.dedup();

                        let mut decoded = None;
                        for enc in candidates {
                            let (text, _, had_errors) = enc.decode(&buf);
                            if !had_errors {
                                decoded = Some(text.into_owned());
                                break;
                            } else if decoded.is_none() {
                                decoded = Some(text.into_owned());
                            }
                        }
                        decoded.unwrap_or_default()
                    };

                    READ_FILES.insert(path_str.clone(), text.clone());
                    text.chars().count() as u64
                } else {
                    metadata.len()
                };

                let ret = PickFileReturn {
                    path: path_str,
                    size: metadata.len(),
                    text_len: size,
                };
                let json = serde_json::to_string(&ret).map_err(|e| js_error!("{}", e))?;
                Ok(JsValue::String(
                    JsString::from_str(&json).map_err(|e| js_error!("{}", e))?,
                ))
            }
            Ok(None) => Ok(JsValue::undefined()),
            Err(_) => Err(js_error!("Failed to receive file path")),
        }
    }
}

pub fn read_file(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> impl std::future::Future<Output = JsResult<JsValue>> {
    let permission_result = plugin_permission_check(ctx, FILESYSTEM_PERMISSION.to_string());

    let path_arg = args
        .get(0)
        .and_then(|v| v.as_string())
        .ok_or_else(|| js_error!("path required"))
        .map(|s| s.to_std_string_lossy());

    let opts: Option<ReadFileOptions> = args
        .get(1)
        .and_then(|v| v.to_string(ctx).ok())
        .and_then(|s| serde_json::from_str(&s.to_std_string_lossy()).ok());

    async move {
        if let Some(err) = permission_result {
            return Err(err);
        }

        let path = path_arg?;
        ensure_allowed(&path).await?;
        let ReadFileOptions { offset, len, decode_text } = opts.ok_or_else(|| js_error!("options required"))?;

        if decode_text {
            // 直接从缓存中获取文本内容
            if let Some(text) = READ_FILES.get(&path) {
                // 这里根据 offset 和 len 截取字符串的子串
                let text_str = text.value();
                let chars: Vec<char> = text_str.chars().collect();
                let start = offset as usize;
                let end = std::cmp::min(start + len as usize, chars.len());
                if start >= end {
                    return Err(js_error!("Invalid offset or len"));
                }
                let slice: String = chars[start..end].iter().collect();

                Ok(JsValue::String(
                    JsString::from_str(&slice).map_err(|e| js_error!("{}", e))?,
                ))
            } else {
                Err(js_error!("File content not cached"))
            }
        } else {
            let mut file = crate::fs::open_file_cross_platform(&path)
                .await
                .map_err(|_| js_error!("Failed to open file"))?;
            file.seek(SeekFrom::Start(offset))
                .map_err(|_| js_error!("seek failed"))?;

            let mut buf = Vec::with_capacity(len as usize);
            (&mut file)
                .take(len)
                .read_to_end(&mut buf)
                .map_err(|_| js_error!("read failed"))?;

            // 返回 base64 编码的数据
            Ok(JsValue::String(
                JsString::from_str(&general_purpose::STANDARD.encode(buf))
                    .map_err(|e| js_error!("{}", e))?,
            ))
        }
    }
}

pub fn unload_file(
    _this: &JsValue,
    args: &[JsValue],
    _ctx: &mut Context,
) -> JsResult<JsValue> {
    let path = args
        .get(0)
        .and_then(|v| v.as_string())
        .ok_or_else(|| js_error!("path required"))?
        .to_std_string_lossy();
    let _ = READ_FILES.remove(&path);
    if ALLOWED_PATHS.remove(&path).is_some() {
        Ok(JsValue::Boolean(true))
    } else {
        Ok(JsValue::Boolean(false))
    }
}

pub fn register_filesystem(global: &mut ObjectInitializer) -> Result<(), String> {
    let ctx = global.context();
    let fs_obj = ObjectInitializer::new(ctx)
        .function(
            NativeFunction::from_async_fn(pick_file_dialog),
            js_string!("pickFile"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(unload_file),
            js_string!("unloadFile"),
            1,
        )
        .function(
            NativeFunction::from_async_fn(read_file),
            js_string!("readFile"),
            2,
        )
        .build();

    global.property(js_string!("filesystem"), fs_obj, Attribute::READONLY);
    Ok(())
}

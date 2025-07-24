use std::{collections::HashMap, future::ready, pin::Pin, str::FromStr};

use base64::{engine::general_purpose, Engine};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsString, JsValue, NativeFunction};
use tokio::sync::oneshot;

pub static NETWORK_PERMISSION: &str = "network";

use crate::pluginsystem::{apis::models::FetchOptions, utils::plugin_permission_check};

pub fn fetch(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> Pin<Box<dyn std::future::Future<Output = boa_engine::JsResult<JsValue>> + 'static>> {

    if let Some(err) = plugin_permission_check(ctx, NETWORK_PERMISSION.to_string()) {
        return Box::pin(ready(Err(err)));
    }

    let url_res: Result<String, boa_engine::JsError> = match args.get(0) {
        Some(v) => v
            .to_string(ctx)
            .map(|s| s.to_std_string_lossy())
            .map_err(|e| js_error!("{}", e)),
        None => Err(js_error!("fetch args[0] (url) is required")),
    };

    let options_res: Result<FetchOptions, boa_engine::JsError> = match args.get(1) {
        Some(v) => match v.to_string(ctx) {
            Ok(s) => serde_json::from_str(&s.to_std_string_lossy())
                .map_err(|e| js_error!("{}", e)),
            Err(e) => Err(js_error!("{}", e)),
        },
        None => Err(js_error!("fetch args[1] (options) is required")),
    };

    let ctx_ptr: *mut Context = ctx;

    Box::pin(async move {
        let url = url_res?;
        let options = options_res?;

        let (tx, rx) = oneshot::channel();

        tauri::async_runtime::spawn(async move {
            let client = crate::net::default_client();
            let mut req = client.request(options.method.unwrap_or("GET".to_string()).parse().unwrap_or(reqwest::Method::GET), &url);

            for (k, v) in &options.headers {
                req = req.header(k, v);
            }
            if !options.body.is_empty() {
                if options.body_encoded {
                    if let Ok(body) = general_purpose::STANDARD.decode(&options.body) {
                        req = req.body(body);
                    }
                }else{
                    req = req.body(options.body.clone());
                }
            }

            let _ = tx.send(req.send().await);
        });

        let resp_result = rx.await.map_err(|e| {
            boa_engine::JsError::from_opaque(JsValue::from(JsString::from(e.to_string())))
        })?;

        match resp_result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers_map: HashMap<_, _> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let bytes = resp.bytes().await.map_err(|e| js_error!("{}", e))?;
                let body = if options.raw{
                    general_purpose::STANDARD.encode(&bytes)
                } else {
                    String::from_utf8_lossy(&bytes).to_string()
                };

                let ctx: &mut Context = unsafe { &mut *ctx_ptr };
                let mut obj = boa_engine::object::ObjectInitializer::new(ctx);
                obj.property(js_string!("status"), status, Attribute::all());
                let headers_json =
                    serde_json::to_string(&headers_map).map_err(|e| js_error!("{}", e))?;
                obj.property(
                    js_string!("headers"),
                    JsString::from_str(&headers_json).map_err(|e| js_error!("{}", e))?,
                    Attribute::all(),
                );
                obj.property(
                    js_string!("contentType"),
                    JsString::from_str(&content_type).map_err(|e| js_error!("{}", e))?,
                    Attribute::all(),
                );
                obj.property(
                    js_string!("body"),
                    JsString::from_str(&body).map_err(|e| js_error!("{}", e))?,
                    Attribute::all(),
                );
                Ok(JsValue::Object(obj.build()))
            }
            Err(e) => {
                let ctx: &mut Context = unsafe { &mut *ctx_ptr };
                let err_obj = boa_engine::object::ObjectInitializer::new(ctx)
                    .property(
                        js_string!("error"),
                        JsString::from_str(&e.to_string()).map_err(|e| js_error!("{}", e))?,
                        Attribute::all(),
                    )
                    .build();
                Ok(JsValue::Object(err_obj))
            }
        }
    })
}

pub fn register_network(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_async_fn(fetch), js_string!("fetch"), 2)
        .build();

    global.property(js_string!("network"), jsobj, Attribute::READONLY);

    Ok(())
}
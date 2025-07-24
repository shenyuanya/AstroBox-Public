use std::future::Future;

use base64::{engine::general_purpose, Engine};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsValue, NativeFunction, JsError};

use crate::pluginsystem::utils::plugin_permission_check;

pub static DEBUG_PERMISSION: &str = "debug";

pub fn send_raw(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    let parsed: Result<Vec<u8>, JsError> = (|| {
        let raw_b64 = args
            .get(0)
            .ok_or_else(|| js_error!("sendRaw args[0] is missing"))?
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy();
        let payload = general_purpose::STANDARD
            .decode(raw_b64)
            .map_err(|_| js_error!("sendRaw args[0] data is invalid"))?;
        Ok(payload)
    })();

    let permission_result = plugin_permission_check(ctx, DEBUG_PERMISSION.to_string());

    Box::pin(async move {

        if let Some(err) = permission_result {
            return Err(err);
        }

        let payload = parsed?;
        crate::miwear::with_connected_device_async(|dev| async move {
            dev.send(payload).await?;
            anyhow::Ok(())
        })
        .await
        .map_err(|e| js_error!("{}", e))?;
        Ok(JsValue::undefined())
    })
}

pub fn register_debug(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_async_fn(send_raw), js_string!("sendRaw"), 1)
        .build();

    global.property(js_string!("debug"), jsobj, Attribute::READONLY);
    
    Ok(())
}
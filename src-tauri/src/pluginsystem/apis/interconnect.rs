use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsError, JsNativeError, JsResult, JsValue, NativeFunction};
use std::future::Future;

use crate::pluginsystem::utils::plugin_permission_check;

pub static INTERCONNECT_PERMISSION: &str = "interconnect";

pub fn send(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    let parsed: Result<(String, String), JsError> = (|| {
        let pkgname_arg = args
            .get(0)
            .ok_or_else(|| js_error!("sendQAICMessage args[0] pkgName missing"))?;
        let data_arg = args
            .get(1)
            .ok_or_else(|| js_error!("sendQAICMessage args[1] data missing"))?;

        let pkgname = pkgname_arg
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy();
        let data = data_arg
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy();

        Ok((pkgname, data))
    })();

    let permission_result = plugin_permission_check(ctx, INTERCONNECT_PERMISSION.to_string());

    Box::pin(async move {

        if let Some(err) = permission_result {
            return Err(err);
        }

        let (pkgname, data) = parsed?;
        let res = crate::miwear::with_connected_device_async(async |dev| {
            crate::miwear::device::thirdpartyapp::send_inter_packet(dev, &pkgname, &data).await?;
            anyhow::Ok(())
        })
        .await;
        if let Err(e) = res {
            let js_error = JsError::from(JsNativeError::typ().with_message(format!("{}", e)));
            return Err(js_error);
        }

        Ok(JsValue::undefined())
    })
}

pub fn register_interconnect(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_async_fn(send), js_string!("sendQAICMessage"), 2)
        .build();

    global.property(js_string!("interconnect"), jsobj, Attribute::READONLY);
    
    Ok(())
}
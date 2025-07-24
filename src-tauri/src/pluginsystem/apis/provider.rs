use std::{future::Future, sync::Arc};
use crate::{community::provider::{jsplugin::JSPluginProvider, Provider}, pluginsystem::utils::plugin_permission_check};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsValue, NativeFunction, JsError};

pub static PROVIDER_PERMISSION: &str = "provider";

pub fn register_community_provider(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {

    let parsed: Result<JSPluginProvider, JsError> = (|| {
        let provider_arg = args
            .get(0)
            .ok_or_else(|| js_error!("registerCommunityProvider args[0] provider is missing"))?;

        let provider: JSPluginProvider = serde_json::from_str(
            &provider_arg
                .to_string(ctx)
                .map_err(|e| js_error!("{}", e))?
                .to_std_string_lossy(),
        )
        .map_err(|e| js_error!("{}", e))?;

        Ok(provider)
    })();

    let permission_result = plugin_permission_check(ctx, PROVIDER_PERMISSION.to_string());

    Box::pin(async move {

        if let Some(err) = permission_result {
            return Err(err);
        }

        let provider = parsed?;
        crate::community::add_provider(Arc::new(provider) as Arc<dyn Provider>).await;
        Ok(JsValue::undefined())
    })
}

pub fn register_provider(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_async_fn(register_community_provider), js_string!("registerCommunityProvider"), 1)
        .build();

    global.property(js_string!("provider"), jsobj, Attribute::READONLY);
    
    Ok(())
}
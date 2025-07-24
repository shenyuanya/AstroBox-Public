use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsString, JsValue, NativeFunction};
use std::{collections::HashMap, str::FromStr};

use crate::pluginsystem::utils::plugin_permission_check;

pub static CONFIG_PERMISSION: &str = "config";

pub fn read_config(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, CONFIG_PERMISSION.to_string()) {
        return Err(err);
    }

    let conf_json = serde_json::to_string(&crate::config::read(|c| {
        let plugin_name = crate::pluginsystem::utils::get_plugin_name(ctx);
        c.plugin_configs.get(&plugin_name).cloned()
    }))
    .map_err(|e| js_error!("{}", e))?;
    Ok(JsValue::String(
        JsString::from_str(&conf_json).map_err(|e| js_error!("{}", e))?,
    ))
}

pub fn write_config(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, CONFIG_PERMISSION.to_string()) {
        return Err(err);
    }

    if let Some(content) = args.get(0) {
        let new_conf: HashMap<String, String> = serde_json::from_str(
            &content
                .to_string(ctx)
                .map_err(|e| js_error!("{}", e))?
                .to_std_string_lossy(),
        )
        .map_err(|e| js_error!("{}", e))?;
        crate::config::write(|c| {
            if let Some(plugc) = c
                .plugin_configs
                .get_mut(&crate::pluginsystem::utils::get_plugin_name(ctx))
            {
                plugc.clear();
                plugc.extend(new_conf);
            }
        });

        return Ok(JsValue::undefined());
    }
    Err(js_error!("writeConfig function args invalid"))
}

pub fn register_config(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(read_config), js_string!("readConfig"), 0)
        .function(NativeFunction::from_fn_ptr(write_config), js_string!("writeConfig"), 1)
        .build();

    global.property(js_string!("config"), jsobj, Attribute::READONLY);
    
    Ok(())
}
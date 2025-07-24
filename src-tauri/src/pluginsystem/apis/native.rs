use std::str::FromStr;

use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsString, JsValue, NativeFunction};

use crate::pluginsystem::utils::plugin_permission_check;

pub static NATIVE_PERMISSION: &str = "native";

pub fn register_function_to_native(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, NATIVE_PERMISSION.to_string()) {
        return Err(err);
    }

    let id =
        "AstroBoxPluginNativeFunctionJSBinding_".to_string() + &crate::tools::random_string(20);
    let func_val = args
        .get(0)
        .ok_or_else(|| js_error!("regNativeFun args[0] is missing"))?;
    let func = func_val
        .as_function()
        .ok_or_else(|| js_error!("regNativeFun function args[0] expected a function"))?
        .clone();

    let _ = crate::pluginsystem::with_plugin_manager_sync(|pm| {
        let plugname = crate::pluginsystem::utils::get_plugin_name(ctx);
        pm.set_plugin_data(&plugname, |data| {
            data.registered_functions.insert(id.clone(), func);
        })
    })
    .map_err(|e| js_error!("{}", e));

    Ok(JsValue::String(
        JsString::from_str(&id).map_err(|e| js_error!("{}", e))?,
    ))
}

pub fn register_native(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(register_function_to_native), js_string!("regNativeFun"), 1)
        .build();

    global.property(js_string!("native"), jsobj, Attribute::READONLY);
    
    Ok(())
}
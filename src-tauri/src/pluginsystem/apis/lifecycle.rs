use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsValue, NativeFunction};

use crate::pluginsystem::utils::plugin_permission_check;

pub static LIFECYCLE_PERMISSION: &str = "lifecycle";

pub fn on_load(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, LIFECYCLE_PERMISSION.to_string()) {
        return Err(err);
    }

    if let Some(cb_val) = args.get(0) {
        let cb_fun = cb_val
            .as_function()
            .ok_or_else(|| js_error!("onLoad function args[0] expected a function"))?
            .clone();

        let _ = crate::pluginsystem::with_plugin_manager_sync(|pm| {
            pm.set_plugin_data(&crate::pluginsystem::utils::get_plugin_name(ctx), |data| {
                data.on_load_function = Some(cb_fun)
            })
        })
        .map_err(|e| js_error!("{}", e));

        return Ok(JsValue::undefined());
    }

    Err(js_error!("onLoad function args invalid"))
}

pub fn register_lifecycle(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(on_load), js_string!("onLoad"), 1)
        .build();

    global.property(js_string!("lifecycle"), jsobj, Attribute::READONLY);
    
    Ok(())
}
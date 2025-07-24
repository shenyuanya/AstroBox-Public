use boa_engine::{
    js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult,
    JsValue, NativeFunction,
};

use crate::pluginsystem::utils::plugin_permission_check;

pub static EVENT_PERMISSION: &str = "event";

pub fn add_event_listener(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    if let Some(err) = plugin_permission_check(ctx, EVENT_PERMISSION.to_string()) {
        return Err(err);
    }

    if let Some(event_name) = args.get(0) {
        if let Some(cb) = args.get(1) {
            let cb_fun = cb
                .as_function()
                .ok_or_else(|| js_error!("addEventListener function args[1] expected a function"))?
                .clone();

            crate::pluginsystem::with_plugin_manager_sync(|pm| {
                if let Ok(name) = event_name.to_string(ctx) {
                    pm.set_plugin_data(&crate::pluginsystem::utils::get_plugin_name(ctx), |data| {
                        data.event_listeners
                            .insert(name.to_std_string_lossy(), cb_fun);
                    })
                    .unwrap_or_else(|e| log::error!("{}", e));
                }
            })
            .unwrap_or_else(|e| log::error!("{}", e));

            return Ok(JsValue::undefined());
        }
    }

    Err(js_error!("addEventListener function args invalid"))
}

pub fn remove_event_listener(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    if let Some(err) = plugin_permission_check(ctx, EVENT_PERMISSION.to_string()) {
        return Err(err);
    }

    let event_name = args
        .get(0)
        .ok_or_else(|| js_error!("removeEventListener requires 1 argument"))?
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    crate::pluginsystem::with_plugin_manager_sync(|pm| {
        pm.set_plugin_data(&crate::pluginsystem::utils::get_plugin_name(ctx), |data| {
            data.event_listeners.remove(&event_name);
        })
        .unwrap_or_else(|e| log::error!("{}", e));
    })
    .unwrap_or_else(|e| log::error!("{}", e));

    Ok(JsValue::undefined())
}

pub fn send_event(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    if let Some(err) = plugin_permission_check(ctx, EVENT_PERMISSION.to_string()) {
        return Err(err);
    }

    if let Some(event_name) = args.get(0) {
        if let Some(payload) = args.get(1) {
            let target = event_name
                .to_string(ctx)
                .map_err(|e| js_error!("{}", e))?
                .to_std_string_lossy();
            crate::pluginsystem::with_plugin_manager_sync(|pm| {
                if let Some(plugin) = pm.get(&crate::pluginsystem::utils::get_plugin_name(ctx)) {
                    for listener in &plugin.js_env_data.event_listeners {
                        if listener.0.to_string() == target {
                            if let Err(e) =
                                listener
                                    .1
                                    .call(&JsValue::Undefined, &[payload.clone()], ctx)
                            {
                                log::error!("event callback error: {}", e);
                            }
                        }
                    }
                    ctx.run_jobs();
                }
            })
            .unwrap_or_else(|e| log::error!("{}", e));

            return Ok(JsValue::undefined());
        }
    }

    Err(js_error!("sendEvent function args invalid"))
}

pub fn register_event(global: &mut ObjectInitializer) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(
            NativeFunction::from_fn_ptr(send_event),
            js_string!("sendEvent"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(add_event_listener),
            js_string!("addEventListener"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(remove_event_listener),
            js_string!("removeEventListener"),
            1,
        )
        .build();

    global.property(js_string!("event"), jsobj, Attribute::READONLY);

    Ok(())
}

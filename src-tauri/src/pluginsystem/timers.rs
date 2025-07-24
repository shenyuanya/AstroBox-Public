use std::{
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

use boa_engine::{
    js_error, js_string, Context, JsResult, JsValue, NativeFunction,
};

use super::utils::get_plugin_name;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub fn register(ctx: &mut Context) -> Result<(), String> {
    ctx.register_global_callable(
        js_string!("setTimeout"),
        2,
        NativeFunction::from_fn_ptr(set_timeout),
    )
    .map_err(|e| e.to_string())?;
    ctx.register_global_callable(
        js_string!("clearTimeout"),
        1,
        NativeFunction::from_fn_ptr(clear_timeout),
    )
    .map_err(|e| e.to_string())?;
    ctx.register_global_callable(
        js_string!("setInterval"),
        2,
        NativeFunction::from_fn_ptr(set_interval),
    )
    .map_err(|e| e.to_string())?;
    ctx.register_global_callable(
        js_string!("clearInterval"),
        1,
        NativeFunction::from_fn_ptr(clear_interval),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn set_timeout(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    create_timer(false, args, ctx)
}
fn set_interval(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    create_timer(true, args, ctx)
}
fn clear_timeout(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    clear_timer(false, args);
    Ok(JsValue::undefined())
}
fn clear_interval(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    clear_timer(true, args);
    Ok(JsValue::undefined())
}

fn create_timer(repeat: bool, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let cb = args
        .get(0)
        .and_then(|v| v.as_function())
        .ok_or_else(|| js_error!("1st arg must be function"))?
        .clone();
    let delay = args
        .get(1)
        .and_then(|v| v.as_number())
        .unwrap_or(0.0)
        .max(0.0) as u64;

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let plugin_name = get_plugin_name(ctx);

    let found = crate::pluginsystem::with_plugin_manager_sync(|pm| {
        if let Some(pl) = pm.plugins.get_mut(&plugin_name) {
            if repeat {
                pl.js_env_data.intervals.insert(id, (cb.clone(), delay));
            } else {
                pl.js_env_data.timeouts.insert(id, cb.clone());
            }
            true
        } else {
            false
        }
    })
    .map_err(|e| js_error!("{}", e))?;

    if !found {
        return Err(js_error!("plugin not found"));
    }

    thread::spawn(move || {
        if repeat {
            loop {
                thread::sleep(Duration::from_millis(delay));
                super::dispatch_timer(&plugin_name, id, true);
            }
        } else {
            thread::sleep(Duration::from_millis(delay));
            super::dispatch_timer(&plugin_name, id, false);
        }
    });

    Ok(JsValue::new(id))
}

fn clear_timer(repeat: bool, args: &[JsValue]) {
    if let Some(id) = args.get(0).and_then(|v| v.as_number()).map(|n| n as u32) {
        crate::pluginsystem::with_plugin_manager_sync(|pm| {
            for pl in pm.plugins.values_mut() {
                if repeat {
                    pl.js_env_data.intervals.remove(&id);
                } else {
                    pl.js_env_data.timeouts.remove(&id);
                }
            }
        })
        .ok();
    }
}

use crate::{miwear::device::MiWearState, pluginsystem::{apis::models::GetDeviceListReturn, utils::plugin_permission_check}};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsString, JsValue, NativeFunction};
use std::{future::Future, str::FromStr};

pub static DEVICE_PERMISSION: &str = "device";

pub fn get_device_list(
    _this: &JsValue,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(_ctx, DEVICE_PERMISSION.to_string()) {
        return Err(err);
    }

    let mut results: Vec<GetDeviceListReturn> = vec![];
    let devs = crate::config::read(|c| c.clone().paired_devices);
    for dev in devs {
        results.push(GetDeviceListReturn {
            name: dev.name,
            addr: dev.addr,
        });
    }
    let list_json = serde_json::to_string(&results).map_err(|e| js_error!("{}", e))?;
    Ok(JsValue::String(
        JsString::from_str(&list_json).map_err(|e| js_error!("{}", e))?,
    ))
}

pub fn get_device_state(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, DEVICE_PERMISSION.to_string()) {
        return Err(err);
    }

    let addr_arg = args
        .get(0)
        .ok_or_else(|| js_error!("getDeviceState args[0] addr is missing"))?;
    let addr = addr_arg
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    let devs = crate::config::read(|c| c.clone().paired_devices);
    for dev in devs {
        if dev.addr == addr {
            return Ok(JsValue::String(JsString::from(
                serde_json::to_string(&dev).map_err(|e| js_error!("{}", e))?,
            )));
        }
    }

    return Err(js_error!(
        "getDeviceState: target device {} not found",
        &addr
    ));
}

pub fn modify_device_state(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, DEVICE_PERMISSION.to_string()) {
        return Err(err);
    }

    let addr_arg = args
        .get(0)
        .ok_or_else(|| js_error!("modifyDeviceState args[0] addr is missing"))?;
    let addr = addr_arg
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();
    let state_arg = args
        .get(1)
        .ok_or_else(|| js_error!("modifyDeviceState args[1] state is missing"))?;
    let state: MiWearState = serde_json::from_str(
        &state_arg
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy(),
    )
    .map_err(|e| js_error!("{}", e))?;

    crate::config::write(|c| {
        c.paired_devices.iter_mut().for_each(|dev| {
            if dev.addr == addr {
                *dev = state.clone();
            }
        });
    });
    return Ok(JsValue::undefined());
}

pub fn disconnect_device(
    _this: &JsValue,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {

    let permission_result = plugin_permission_check(_ctx, DEVICE_PERMISSION.to_string());
    
    async move {

        if let Some(err) = permission_result {
            return Err(err);
        }

        crate::miwear::with_connected_device_async(|dev| async move {
            dev.disconnect().await?;
            anyhow::Ok(())
        })
        .await
        .map_err(|e| js_error!("{}", e))?;
        Ok(JsValue::undefined())
    }
}

pub fn register_device(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(get_device_list), js_string!("getDeviceList"), 0)
        .function(NativeFunction::from_fn_ptr(get_device_state), js_string!("getDeviceState"), 1)
        .function(NativeFunction::from_fn_ptr(modify_device_state), js_string!("modifyDeviceState"), 2)
        .function(NativeFunction::from_async_fn(disconnect_device), js_string!("disconnectDevice"), 0)
        .build();

    global.property(js_string!("device"), jsobj, Attribute::READONLY);
    
    Ok(())
}
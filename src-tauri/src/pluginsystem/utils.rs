use boa_engine::{js_error, js_string, Context, JsError};

use crate::pluginsystem::apis;

pub fn get_plugin_name(ctx: &mut Context) -> String {
    ctx
        .global_object()
        .get(js_string!("PLUGIN_NAME"), ctx)
        .ok()
        .and_then(|v| v.as_string().map(|s| s.to_std_string_lossy()))
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(debug_assertions)]
pub fn is_debug_version() -> bool {
    return true;
}

#[cfg(not(debug_assertions))]
pub fn is_debug_version() -> bool {
    return false;
}

pub fn plugin_permission_check(ctx: &mut Context, permission: String) -> Option<JsError> {
    
    let mut result = false;
    let mut debug = false;
    let mut checked = false;

    crate::pluginsystem::with_plugin_manager_sync(|pm| {
        result = match pm.get(&get_plugin_name(ctx)) {
            Some(plugin) => {
                // 如果拥有debug权限那就不进行其他权限的检查
                debug = plugin.manifest.permissions.contains(&apis::debug::DEBUG_PERMISSION.to_string());
                checked = plugin.manifest.permissions.contains(&permission);

                debug || checked
            },
            None => { false },
        }
    })
    .unwrap_or_else(|e| log::error!("{}", e));

    if (permission == "debug" && !is_debug_version()) || (!checked && debug && !is_debug_version()) {
        return Some(js_error!("debug permission are disabled in release build"))
    }

    if result {
        return None
    }

    return Some(js_error!("permission denied: {}", permission))
}
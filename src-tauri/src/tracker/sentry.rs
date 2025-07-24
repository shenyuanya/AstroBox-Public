use tauri_plugin_sentry::sentry::{self, ClientInitGuard};

pub fn init_sentry() -> ClientInitGuard {
    let client = sentry::init((
        "__OPENSOURCE__DELETED__",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            auto_session_tracking: true,
            send_default_pii: true,
            ..Default::default()
        },
    ));
    
    #[cfg(not(target_os = "ios"))]
    let _guard = sentry_rust_minidump::init(&(*client).clone());

    client
}

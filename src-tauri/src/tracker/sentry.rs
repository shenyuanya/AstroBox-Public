use tauri_plugin_sentry::sentry::{self, ClientInitGuard};

pub fn init_sentry() -> ClientInitGuard {
    let client = sentry::init((
        "https://7f2556c98fdac27cb73b980994bd547b@o4509616507846656.ingest.us.sentry.io/4509616509353984",
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

use once_cell::sync::Lazy;
use tauri::Emitter;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub mod bleuuids;
pub mod btrecv;
pub mod device;
pub mod network_stack;
pub mod packet;
pub mod command_pool;

pub use device::{MiWearDevice, SecurityKeys};

pub static CONNECTED_DEVICE: Lazy<RwLock<Option<Arc<MiWearDevice>>>> =
    Lazy::new(|| tokio::sync::RwLock::new(None));

pub const DISCONNECT_EVENT: &str = "device-disconnected";

static DISCONNECT_TX: Lazy<broadcast::Sender<()>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(16);
    tx
});

pub async fn set_connected_device(dev: Option<Arc<MiWearDevice>>) {
    *CONNECTED_DEVICE.write().await = dev;
}

pub fn subscribe_disconnect() -> broadcast::Receiver<()> {
    DISCONNECT_TX.subscribe()
}

pub fn notify_disconnect() {
    let _ = DISCONNECT_TX.send(());
    if let Some(app) = crate::APP_HANDLE.get() {
        let _ = app.emit(DISCONNECT_EVENT, ());
    }
}

pub async fn with_connected_device<F, R>(f: F) -> Option<R>
where
    F: FnOnce(Arc<MiWearDevice>) -> R,
{
    let guard = CONNECTED_DEVICE.read().await;
    guard.as_ref().cloned().map(f)
}

pub async fn with_connected_device_async<F, Fut, R, E>(f: F) -> Result<R, String>
where
    F: FnOnce(Arc<MiWearDevice>) -> Fut,
    Fut: std::future::Future<Output = Result<R, E>>,
    E: ToString,
{
    let dev = {
        let guard = CONNECTED_DEVICE.read().await;
        guard.as_ref().cloned()
    };

    let dev = match dev {
        Some(dev) => dev,
        None => return Err("No devices are connected".to_owned()),
    };

    f(dev).await.map_err(|e| e.to_string())
}

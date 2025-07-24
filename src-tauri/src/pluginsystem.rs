use anyhow::{anyhow, Result};
use crossbeam::channel;
use once_cell::sync::OnceCell;
use std::{cell::RefCell, path::PathBuf, thread};
use tokio::sync::oneshot;
use boa_engine::JsValue;
use manager::PluginManager;

pub mod apis;
pub mod console;
pub mod globals;
pub mod manager;
pub mod manifest;
pub mod plugin;
pub mod utils;
pub mod timers;

// ---------- 线程局部存 PluginManager，并记录插件线程 id ----------
thread_local! {
    static PM_IN_THREAD: RefCell<Option<*mut PluginManager>> = const { RefCell::new(None) };
}
static PLUGIN_THREAD_ID: OnceCell<thread::ThreadId> = OnceCell::new();

// ---------- channel 仍然只装 Send 闭包 ----------
enum Command {
    Exec(Box<dyn FnOnce(&mut PluginManager) + Send + 'static>),
}
static PLUGIN_TX: OnceCell<channel::Sender<Command>> = OnceCell::new();

pub fn init(dir: PathBuf) -> Result<()> {
    let (tx, rx) = channel::unbounded::<Command>();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to build runtime: {e}");
                return;
            }
        };

        runtime.block_on(async move {
            let mut pm = PluginManager::new();

            tokio::task::block_in_place(|| {
                PM_IN_THREAD.with(|cell| *cell.borrow_mut() = Some(&mut pm as *mut _));
                // 初始化插件线程 ID，如已设置则记录错误
                if PLUGIN_THREAD_ID.set(thread::current().id()).is_err() {
                    log::error!("PLUGIN_THREAD_ID 已设置");
                }
            });

            if let Err(e) = pm.load_from_dir(dir) {
                log::error!("PluginManager init failed: {e}");
                return;
            }

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    Command::Exec(task) => {
                        task(&mut pm);
                    }
                }
            }
        });
    });

    PLUGIN_TX
        .set(tx)
        .map_err(|_| anyhow!("Plugin system already initialised"))
}

pub fn dispatch_timer(plugin_name: &str, id: u32, is_interval: bool) {
    if let Some(tx) = PLUGIN_TX.get() {
        let plugin_name = plugin_name.to_owned();
        let _ = tx.send(Command::Exec(Box::new(move |pm| {
            if let Some(pl) = pm.plugins.get_mut(&plugin_name) {
                let ctx = &mut pl.js_context;

                if is_interval {
                    if let Some((cb, _delay)) = pl.js_env_data.intervals.get(&id) {
                        let _ = cb.call(&JsValue::Undefined, &[], ctx);
                    }
                } else if let Some(cb) = pl.js_env_data.timeouts.remove(&id) {
                    let _ = cb.call(&JsValue::Undefined, &[], ctx);
                }

                let _ = ctx.run_jobs(); // 处理 Promise micro-tasks
            }
        })));
    }
}

// ---------- 同线程：同步 API，无 Send 约束 ----------
pub fn with_plugin_manager_sync<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut PluginManager) -> R,
{
    // 断言：一定在插件线程
    debug_assert_eq!(
        Some(thread::current().id()),
        PLUGIN_THREAD_ID.get().copied(),
        "with_plugin_manager_sync must be called from the plugin thread"
    );

    // SAFETY: 只在插件线程里解引用 TLS 指针
    unsafe {
        PM_IN_THREAD.with(|cell| {
            let pm_ptr = cell
                .borrow()
                .ok_or_else(|| anyhow!("PluginManager TLS not set"))? as *mut PluginManager;
            Ok(f(&mut *pm_ptr))
        })
    }
}

// ---------- 跨线程：异步 API，闭包必须 Send ----------
pub async fn with_plugin_manager_async<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut PluginManager) -> R + Send + 'static,
    R: Send + 'static,
{
    // 快速路径：如果本来就在插件线程，也直接同步调用（这样少一次 channel hop）
    if Some(thread::current().id()) == PLUGIN_THREAD_ID.get().copied() {
        return with_plugin_manager_sync(f);
    }

    let (tx, rx) = oneshot::channel();
    let cmd = Command::Exec(Box::new(move |pm| {
        let _ = tx.send(f(pm));
    }));

    PLUGIN_TX
        .get()
        .ok_or_else(|| anyhow!("Plugin system not initialised"))?
        .send(cmd)
        .map_err(|_| anyhow!("Plugin thread unexpectedly closed"))?;

    rx.await.map_err(|_| anyhow!("Plugin thread dropped the response"))
}

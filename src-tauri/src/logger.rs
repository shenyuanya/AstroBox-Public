use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use once_cell::sync::{Lazy, OnceCell};
use std::{fs, io};
use tauri::{Emitter, Manager};
use crossbeam::channel;

static START_TIME: Lazy<String> =
    Lazy::new(|| Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());

pub const LOG_EVENT: &str = "backend-log";

static LOG_TX: OnceCell<channel::Sender<String>> = OnceCell::new();
static LOG_FILE_PATH: OnceCell<String> = OnceCell::new();

pub fn current_log_path() -> Option<&'static str> {
    LOG_FILE_PATH.get().map(|s| s.as_str())
}

pub fn read_current_log() -> Option<String> {
    current_log_path().and_then(|p| fs::read_to_string(p).ok())
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // 构造日志文件夹路径
    let base_path = format!(
        "{}/rslogs",
        crate::APP_HANDLE
            .get()
            .ok_or_else(|| anyhow::anyhow!("APP_HANDLE 未初始化"))?
            .path()
            .app_log_dir()
            .map_err(|_| anyhow::anyhow!("无法获取日志目录"))?
            .to_string_lossy()
    );
    fs::create_dir_all(&base_path)?;
    let file_path = format!("{}/{}.log", base_path, &*START_TIME);
    LOG_FILE_PATH.set(file_path.clone()).ok();

    let (tx, rx) = channel::unbounded::<String>();
    LOG_TX.set(tx.clone()).ok();
    std::thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if let Some(app) = crate::APP_HANDLE.get() {
                let _ = app.emit(LOG_EVENT, line.clone());
            }
        }
    });

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);
    struct EventWriter {
        tx: channel::Sender<String>,
    }

    impl io::Write for EventWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if let Ok(s) = std::str::from_utf8(buf) {
                let _ = self.tx.send(s.trim_end_matches('\n').to_string());
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }
    #[cfg(debug_assertions)] {
        fern::Dispatch::new()
            .level(LevelFilter::Trace)
            .chain(
                fern::Dispatch::new()
                    .format(move |out, message, record| {
                        out.finish(format_args!(
                            "{ts} {level:<5} [{file}:{line}] {module} – {msg}",
                            ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            level = colors.color(record.level()),
                            file = record.file().unwrap_or("unknown"),
                            line = record.line().unwrap_or(0),
                            module = record.module_path().unwrap_or("unknown"),
                            msg = message,
                        ))
                    })
                    .chain(io::stdout()),
            )
            .chain(
                fern::Dispatch::new()
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "{ts} {level:<5} [{file}:{line}] {module} - {msg}",
                            ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            level = record.level(),
                            file = record.file().unwrap_or("unknown"),
                            line = record.line().unwrap_or(0),
                            module = record.module_path().unwrap_or("unknown"),
                            msg = message,
                        ))
                    })
                    .chain(fern::log_file(&file_path)?),
            )
            .chain(Box::new(EventWriter { tx: tx.clone() }) as Box<dyn io::Write + Send>)
            .apply()?;
    }

    #[cfg(not(debug_assertions))] {
        fern::Dispatch::new()
            .level(LevelFilter::Info)
            .chain(
                fern::Dispatch::new()
                    .format(move |out, message, record| {
                        out.finish(format_args!(
                            "{ts} {level:<5} [{file}:{line}] {module} – {msg}",
                            ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            level = colors.color(record.level()),
                            file = record.file().unwrap_or("unknown"),
                            line = record.line().unwrap_or(0),
                            module = record.module_path().unwrap_or("unknown"),
                            msg = message,
                        ))
                    })
                    .chain(io::stdout()),
            )
            .chain(
                fern::Dispatch::new()
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "{ts} {level:<5} [{file}:{line}] {module} - {msg}",
                            ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            level = record.level(),
                            file = record.file().unwrap_or("unknown"),
                            line = record.line().unwrap_or(0),
                            module = record.module_path().unwrap_or("unknown"),
                            msg = message,
                        ))
                    })
                    .chain(fern::log_file(&file_path)?),
            )
            .chain(Box::new(EventWriter { tx }) as Box<dyn io::Write + Send>)
            .apply()?;
    }

    log::info!("Logger initialization completed.");

    Ok(())
}

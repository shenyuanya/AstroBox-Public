use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use once_cell::sync::Lazy;
use std::{fs, io};
use tauri::Manager;

static START_TIME: Lazy<String> =
    Lazy::new(|| Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());

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

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);
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
            .apply()?;
    }

    log::info!("Logger initialization completed.");

    Ok(())
}

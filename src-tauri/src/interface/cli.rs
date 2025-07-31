use crate::{interface, miwear::device::resutils::get_file_type};
use serde::Serialize;
use url::Url;
use std::path::Path;
use tauri::{AppHandle, Emitter};

fn is_path_exists(s: &str) -> bool {
    Path::new(s).exists()
}

pub async fn handle(app: AppHandle, args: &[String]) -> bool {
    match args[0].as_str() {
        /* "help" => {
            let version = env!("CARGO_PKG_VERSION");
            println!("AstroBox CLI v{version}");
            if args.len() == 1 {
                println!("Available commands:");
                println!("  help <command> - Show the help messages");
                println!("  exit - Exit the program");
                println!("  install <package> [options] - Install a package");
                return;
            }
            match args[1].as_str() {
                "help" => {
                    println!("Show the help messages");
                    println!("Usage: astrobox <command>");
                }
                "exit" => {
                    println!("Exit the program");
                }
                "install" => {
                    println!("Install a package");
                    println!("Usage: astrobox install <package> [--type <type>]");
                    println!("  <package> - The package name");
                    println!("  --type <type> - The package type(watchface, quickapp ,firmware)");
                }
                _ => {
                    println!("Unknown command: {}", args[1]);
                }
            }
        } */
        "exit" => {
            std::process::exit(0);
        }
        "install"|"i" => {
            let package = &args[1];
            let tail = &args[2..];
            if package == "" {
                log::info!("Please specify a package");
                return true;
            }
            let mut package_type = "unset";
            for i in 0..tail.len() {
                if tail[i] == "--type" {
                    if i + 1 < tail.len() {
                        package_type = &tail[i + 1];
                    }
                }
            }
            if package_type == "unset" {
                match get_file_type(package).await.unwrap_or_default().as_str() {
                    "watchface" => {
                        package_type = "watchface";
                    }
                    "quickapp" => {
                        package_type = "quickapp";
                    }
                    _ => {
                        log::error!("Unknown package type");
                        return true;
                    }
                }
            }
            log::info!("Installing package {} with type {}", package, package_type);
            if let Err(e) = app.emit("open_file", FileOpen { path: package.to_string(), file_type: package_type.to_string() }) {
                log::error!("文件打开事件发送失败: {}", e);
            }
            if let Err(e) = app.emit("start_install",{}) {
                log::error!("安装事件发送失败: {}", e);
            }
            true
        }
        _ => {
            if args[0].starts_with("astrobox://") {
                if let std::result::Result::Ok(url) = Url::parse(&args[0]) {
                    interface::deeplink::handle_link(url);
                }
                return true;
            }
            if is_path_exists(&args[0]) {
                log::info!("open file {}", &args[0]);
                let path = args[0].clone();
                let file_type = get_file_type(&*path).await.unwrap_or_default();
                if let Err(e) = app.emit("open_file", FileOpen { path, file_type }) {
                    log::error!("文件打开事件发送失败: {}", e);
                }
                return true;
            }
            println!("Unknown command: {}", args[0]);
            true
        }
    }
}

#[derive(Serialize, Clone)]
struct FileOpen {
    pub path: String,
    pub file_type: String,
}

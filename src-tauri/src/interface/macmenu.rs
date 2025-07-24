use tauri::{
    menu::{Menu, MenuBuilder, PredefinedMenuItem, SubmenuBuilder}, AppHandle, Emitter, Runtime
};

pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let app_menu = SubmenuBuilder::new(app, app.package_info().name.clone())
        .text("switch_device", "切换设备")
        .item(&PredefinedMenuItem::quit(app, Some("退出 AstroBox"))?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "编辑")
        .item(&PredefinedMenuItem::copy(app, Some("复制"))?)
        .item(&PredefinedMenuItem::cut(app, Some("剪切"))?)
        .item(&PredefinedMenuItem::paste(app, Some("粘贴"))?)
        .item(&PredefinedMenuItem::select_all(app, Some("全选"))?)
        .item(&PredefinedMenuItem::undo(app, Some("撤销"))?)
        .item(&PredefinedMenuItem::redo(app, Some("重做"))?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "文件")
        .text("open_rpk", "选取并安装快应用")
        .text("open_wf",  "选取并安装表盘")
        .text("open_fw", "选取并安装固件")
        .text("open_abp", "选取并安装 AstroBox 插件")
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "帮助")
        .text("website", "访问官方网站")
        .text("website_docs", "前往文档页")
        .text("about", "关于 AstroBox")
        .build()?;

    MenuBuilder::new(app)
        .items(&[&app_menu, &edit_menu, &file_menu, &help_menu])
        .build()
}

pub fn process_event(app: &AppHandle, name: String) {
    match name.as_str() {
        "website" => {
            let _ = crate::tools::open_url_with_default_browser("https://astrobox.online".to_string());
        },
        _ => {
            if let Err(e) = app.emit(&format!("menubar_{}", name), "") {
                log::error!("菜单事件发送失败: {}", e);
            }
        }
    }
}
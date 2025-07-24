use boa_engine::{Context, JsResult, Trace};
use boa_gc::Finalize;
use boa_runtime::{Console, ConsoleState, Logger};

pub fn register_console(context: &mut Context, plugin_name: &String) {
    let logger = PluginLogger {
        plugin_name: plugin_name.to_string(),
    };

    if let Err(e) = Console::register_with_logger(context, logger) {
        log::error!("Register console failed: {}", e);
    }
}

#[derive(Finalize, Trace)]
struct PluginLogger {
    plugin_name: String,
}

impl Logger for PluginLogger {
    fn log(&self, msg: String, _: &ConsoleState, _: &mut Context) -> JsResult<()> {
        log::info!("[JS Plugin: {}][LOG]: {}", self.plugin_name, msg);
        Ok(())
    }
    fn info(&self, msg: String, _: &ConsoleState, _: &mut Context) -> JsResult<()> {
        log::info!("[JS Plugin: {}][INFO]: {}", self.plugin_name, msg);
        Ok(())
    }
    fn warn(&self, msg: String, _: &ConsoleState, _: &mut Context) -> JsResult<()> {
        log::warn!("[JS Plugin: {}][WARN]: {}", self.plugin_name, msg);
        Ok(())
    }
    fn error(&self, msg: String, _: &ConsoleState, _: &mut Context) -> JsResult<()> {
        log::error!("[JS Plugin: {}][ERROR]: {}", self.plugin_name, msg);
        Ok(())
    }
    fn debug(&self, msg: String, _: &ConsoleState, _: &mut Context) -> JsResult<()> {
        log::error!("[JS Plugin: {}][DEBUG]: {}", self.plugin_name, msg);
        Ok(())
    }
}
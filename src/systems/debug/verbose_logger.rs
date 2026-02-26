use chrono::Local;
use env_logger::Builder;
use log::Level;
use rust_i18n::t;
use std::io::Write;

rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

pub fn init_verbose_logger(level_filter: Option<log::LevelFilter>) {
    let mut builder = match level_filter {
        Some(f) => {
            let mut b = Builder::new();
            b.filter_level(f);
            b
        }
        None => return,
    };

    builder
        .format(|buf, record| {
            let now = Local::now();
            let timestamp = now.format("%y-%-m-%-d %H:%M:%S");
            let level = record.level();
            let args = record.args();

            let (prefix, color_code) = match level {
                Level::Error => (t!("logger.error").trim().to_string(), "\x1b[31m"),
                Level::Warn => (t!("logger.warn").trim().to_string(), "\x1b[33m"),
                Level::Info => (t!("logger.info").trim().to_string(), "\x1b[37m"),
                Level::Debug => (t!("logger.debug").trim().to_string(), "\x1b[90m"),
                Level::Trace => (t!("logger.trace").trim().to_string(), "\x1b[36m"),
            };

            let colored_prefix = if prefix.is_empty() {
                String::new()
            } else {
                format!("{}[{}] {}: \x1b[0m", color_code, timestamp, prefix)
            };

            writeln!(buf, "{}{}", colored_prefix, args)
        })
        .init();
}

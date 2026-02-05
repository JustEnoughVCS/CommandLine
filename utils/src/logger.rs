use std::path::Path;

use colored::Colorize;
use env_logger::{Builder, Target};
use just_enough_vcs::{
    lib::data::vault::vault_config::LoggerLevel, utils::string_proc::format_path::format_path,
};
use log::{Level, LevelFilter};

pub fn build_env_logger(log_path: impl AsRef<Path>, logger_level: LoggerLevel) {
    use std::io::{self, Write};

    struct MultiWriter<A, B> {
        a: A,
        b: B,
    }

    impl<A: Write, B: Write> MultiWriter<A, B> {
        fn new(a: A, b: B) -> Self {
            Self { a, b }
        }
    }

    impl<A: Write, B: Write> Write for MultiWriter<A, B> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let _ = self.a.write(buf);
            self.b.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            let _ = self.a.flush();
            self.b.flush()
        }
    }

    let log_path = {
        let path = log_path.as_ref();
        let Ok(path) = format_path(path) else {
            eprintln!(
                "Build logger failed: {} is not a vaild path.",
                path.display()
            );
            return;
        };
        path
    };

    let mut builder = Builder::new();

    let log_format = |buf: &mut env_logger::fmt::Formatter, record: &log::Record| {
        let now = chrono::Local::now();

        let level_style = match record.level() {
            Level::Error => record.args().to_string().red().bold(),
            Level::Warn => record.args().to_string().yellow().bold(),
            Level::Info => record.args().to_string().white(),
            Level::Debug => record.args().to_string().white(),
            Level::Trace => record.args().to_string().cyan(),
        };

        writeln!(
            buf,
            "{} {}",
            now.format("%H:%M:%S")
                .to_string()
                .truecolor(105, 105, 105)
                .bold(),
            level_style
        )
    };

    let log_file = std::fs::File::create(log_path).expect("Failed to create log file");
    let combined_target = Target::Pipe(Box::new(MultiWriter::new(std::io::stdout(), log_file)));

    let level = match logger_level {
        LoggerLevel::Debug => LevelFilter::Debug,
        LoggerLevel::Trace => LevelFilter::Trace,
        LoggerLevel::Info => LevelFilter::Info,
    };

    builder
        .format(log_format)
        .filter(None, level.clone())
        .filter_module("just_enough_vcs", level)
        .target(combined_target)
        .init();
}

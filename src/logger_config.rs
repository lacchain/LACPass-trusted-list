use std::io::Write;
use std::thread;

use chrono::prelude::*;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use env_logger::Env;
use log::Record;

pub fn setup_logger(log_thread: bool, rust_log: Option<&str>) {
    let _output_format = move |formatter: &mut Formatter, record: &Record| {
        let thread_name = if log_thread {
            format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
        } else {
            "".to_string()
        };

        let local_time: DateTime<Local> = Local::now();
        let time_str = local_time.format("%H:%M:%S%.3f").to_string();
        write!(
            formatter,
            "{} {}{} - {} - {}\n",
            time_str,
            thread_name,
            record.level(),
            record.target(),
            record.args()
        )
    };

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    // builder.format(output_format); // avoiding custom formatting
    rust_log.map(|conf| builder.parse_filters(conf));
    builder.init();
}

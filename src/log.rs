#[cfg(not(debug_assertions))]
use crate::config::get_config_dir;
#[cfg(not(debug_assertions))]
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(debug_assertions)]
pub fn init() {
    let mut cfg_builder = simplelog::ConfigBuilder::new();
    cfg_builder.set_time_format_rfc3339();
    let _ = cfg_builder.set_time_offset_to_local();
    let log_config = cfg_builder.build();
    let _ = simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .or_else(|_| {
        simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default())
    });
}

#[cfg(not(debug_assertions))]
pub fn init() {
    if let Ok(dir) = get_config_dir() {
        let file_appender = tracing_appender::rolling::Builder::new()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix("endgame")
            .filename_suffix("log")
            .max_log_files(10)
            .build(&dir)
            .unwrap_or_else(|_| tracing_appender::rolling::daily(&dir, "endgame.log"));

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .with(tracing_subscriber::EnvFilter::new("info"));

        let _ = subscriber.try_init();

        // Keep the non-blocking writer alive for the lifetime of the process.
        std::mem::forget(guard);
    }
}

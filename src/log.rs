#[cfg(not(debug_assertions))]
use crate::config::get_config_dir;

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
    .or_else(|_| simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()));
}

#[cfg(not(debug_assertions))]
pub fn init() {
    if let Ok(dir) = get_config_dir() {
        if let Ok(file) = std::fs::File::create(dir.join("log.txt")) {
            let mut cfg_builder = simplelog::ConfigBuilder::new();
            cfg_builder.set_time_format_rfc3339();
            let _ = cfg_builder.set_time_offset_to_local();
            let log_config = cfg_builder.build();
            let _ = simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, file);
        }
    }
}

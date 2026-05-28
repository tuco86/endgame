#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod hotkey;
mod log;
mod process;
mod tray;

use crate::config::load_config;
use crate::hotkey::spawn_hotkey_listener;
use crate::tray::{TrayApp, UserEvent};
use process::enable_debug_privilege;
use std::sync::Arc;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::init();
    ::log::info!("Endgame - Emergency Game Killer starting");

    let config = load_config()?;

    if enable_debug_privilege() {
        ::log::info!("SeDebugPrivilege enabled");
    } else {
        ::log::warn!("Could not enable SeDebugPrivilege (may need elevation)");
    }

    let config_arc = Arc::new(config);
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    spawn_hotkey_listener(proxy.clone());
    let mut app = TrayApp::new(config_arc.clone(), proxy)?;

    ::log::info!("Entering event loop");
    event_loop.run_app(&mut app)?;
    ::log::info!("Shutting down");
    Ok(())
}

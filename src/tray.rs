use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;

use crate::config::{self, Config};
use crate::process;

pub struct TrayApp {
    quit_item_id: tray_icon::menu::MenuId,
    config_item_id: tray_icon::menu::MenuId,
    logdir_item_id: tray_icon::menu::MenuId,
    _tray_icon: TrayIcon, // keep handle alive
    hotkey_rx: Option<Receiver<()>>,
    config: Arc<Config>,
}

impl TrayApp {
    pub fn new(config: Arc<Config>) -> Result<Self, Box<dyn std::error::Error>> {
        // Create tray menu
        let config_item = MenuItem::new("Configurations", true, None);
        let logdir_item = MenuItem::new("Open Log Folder", true, None);
        let quit_item = MenuItem::new("Exit", true, None);
        let menu = Menu::new();
        menu.append(&config_item)?;
        menu.append(&logdir_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        // Create tray icon with a red circle
        let icon = create_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Endgame - Ctrl+Alt+End to kill game")
            .with_icon(icon)
            .build()?;

        log::info!("Tray icon created. Right-click to access menu.");
        log::info!("Ready: Ctrl+Alt+End to kill foreground/fullscreen process");

        Ok(TrayApp {
            quit_item_id: quit_item.id().clone(),
            config_item_id: config_item.id().clone(),
            logdir_item_id: logdir_item.id().clone(),
            _tray_icon: tray_icon,
            hotkey_rx: None,
            config,
        })
    }

    pub fn with_hotkey_receiver(mut self, rx: Receiver<()>) -> Self {
        self.hotkey_rx = Some(rx);
        self
    }
}

impl ApplicationHandler for TrayApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Set up a timer to periodically check for menu events
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Handle tray menu events
        if let Ok(menu_event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if menu_event.id == self.quit_item_id {
                log::info!("Exit requested from tray menu");
                event_loop.exit();
            } else if menu_event.id == self.config_item_id {
                match config::ensure_config_file() {
                    Ok(path) => {
                        log::info!("Opening config file: {}", path.display());
                        let _ = Command::new("cmd")
                            .args(["/C", "start", "", &path.display().to_string()])
                            .spawn();
                    }
                    Err(e) => log::error!("Failed to prepare config file: {}", e),
                }
            } else if menu_event.id == self.logdir_item_id {
                match config::get_config_dir() {
                    Ok(dir) => {
                        log::info!("Opening log directory: {}", dir.display());
                        let _ = Command::new("explorer").arg(dir).spawn();
                    }
                    Err(e) => log::error!("Failed to get log directory: {}", e),
                }
            }
        }

        // Check hotkey
        if let Some(rx) = &self.hotkey_rx {
            while let Ok(()) = rx.try_recv() {
                log::info!("Hotkey pressed");
                if let Some(target) = process::find_target(&self.config) {
                    log::info!("Target: {} (PID {})", target.name, target.pid);
                    if process::terminate(target.pid) {
                        log::info!("Terminated {} (PID {})", target.name, target.pid);
                    } else {
                        log::error!("Failed to terminate {} (PID {})", target.name, target.pid);
                    }
                } else {
                    log::warn!("No suitable target found");
                }
            }
        }
    }
}

fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    // Create a simple red circle icon (32x32)
    let size = 32usize;
    let size_u32 = 32u32;
    let mut rgba_data = vec![0u8; size * size * 4];

    let center = size as f32 / 2.0;
    let radius = (size as f32 / 2.0) - 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();

            let idx = (y * size + x) * 4;

            if distance <= radius {
                // Red circle
                rgba_data[idx] = 220; // R
                rgba_data[idx + 1] = 20; // G
                rgba_data[idx + 2] = 20; // B
                rgba_data[idx + 3] = 255; // A
            } else {
                // Transparent background
                rgba_data[idx] = 0; // R
                rgba_data[idx + 1] = 0; // G
                rgba_data[idx + 2] = 0; // B
                rgba_data[idx + 3] = 0; // A (transparent)
            }
        }
    }

    Ok(tray_icon::Icon::from_rgba(rgba_data, size_u32, size_u32)
        .map_err(|e| format!("Icon creation failed: {}", e))?)
}

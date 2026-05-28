use std::process::Command;
use std::sync::Arc;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;

use crate::config::{self, Config};
use crate::process;

#[derive(Debug)]
pub enum UserEvent {
    Hotkey,
    Menu(MenuEvent),
}

pub struct TrayApp {
    quit_item_id: tray_icon::menu::MenuId,
    config_item_id: tray_icon::menu::MenuId,
    logdir_item_id: tray_icon::menu::MenuId,
    _tray_icon: TrayIcon,
    config: Arc<Config>,
}

impl TrayApp {
    pub fn new(
        config: Arc<Config>,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config_item = MenuItem::new("Configurations", true, None);
        let logdir_item = MenuItem::new("Open Log Folder", true, None);
        let quit_item = MenuItem::new("Exit", true, None);
        let menu = Menu::new();
        menu.append(&config_item)?;
        menu.append(&logdir_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let icon = create_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Endgame - Ctrl+Alt+End to kill game")
            .with_icon(icon)
            .build()?;

        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(UserEvent::Menu(event));
        }));

        log::info!("Tray icon created. Right-click to access menu.");
        log::info!("Ready: Ctrl+Alt+End to kill foreground/fullscreen process");

        Ok(TrayApp {
            quit_item_id: quit_item.id().clone(),
            config_item_id: config_item.id().clone(),
            logdir_item_id: logdir_item.id().clone(),
            _tray_icon: tray_icon,
            config,
        })
    }
}

impl ApplicationHandler<UserEvent> for TrayApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
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

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Hotkey => {
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
            UserEvent::Menu(menu_event) => {
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
        }
    }
}

fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    const SIZE: u32 = 32;
    const RGBA: &[u8] = include_bytes!("../assets/tray-32.rgba");

    Ok(tray_icon::Icon::from_rgba(RGBA.to_vec(), SIZE, SIZE)
        .map_err(|e| format!("Icon creation failed: {}", e))?)
}

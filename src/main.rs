#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod dashboard;
mod focus;
mod icon;
mod tray;

use dashboard::Dashboard;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray::{Tray, UserEvent};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::w;
use wry::WebContext;

const TRAY_REFRESH: Duration = Duration::from_secs(5);

/// Two trackers would count every second twice.
fn already_running() -> bool {
    // the handle stays open for the lifetime of the process
    let _ = unsafe { CreateMutexW(None, true, w!("Local\\Tracky.SingleInstance")) };
    (unsafe { GetLastError() }) == ERROR_ALREADY_EXISTS
}

fn main() {
    if already_running() {
        return;
    }

    autostart::enable_on_first_run();

    // WebView2 needs a writable folder; next to the exe (Program Files) it is not
    let data_dir = std::env::var_os("LOCALAPPDATA").map(|p| std::path::PathBuf::from(p).join("Tracky"));
    let mut web_context = WebContext::new(data_dir);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let focus = focus::start();
    let tray = Tray::new(&event_loop, Arc::clone(&focus));
    let mut dashboard: Option<Dashboard> = None;
    let mut next_refresh = Instant::now();
    // started by hand -> show the dashboard; started with Windows -> stay in the tray
    if !std::env::args().any(|a| a == "--background") {
        event_loop.create_proxy().send_event(UserEvent::OpenDashboard).ok();
    }

    event_loop.run(move |event, target, control_flow| {
        match event {
            Event::WindowEvent { window_id, event, .. } => {
                if let Some(d) = dashboard.as_ref().filter(|d| d.id() == window_id) {
                    d.handle_event(&event);
                }
            }
            Event::UserEvent(UserEvent::OpenDashboard) => {
                dashboard.get_or_insert_with(|| Dashboard::new(target, &mut web_context, Arc::clone(&focus))).show();
            }
            Event::UserEvent(UserEvent::Quit) => {
                focus.set_paused(true); // writes the running session
                *control_flow = ControlFlow::Exit;
                return;
            }
            Event::UserEvent(e) => tray.handle_event(e),
            _ => {}
        }

        if Instant::now() >= next_refresh {
            tray.refresh();
            next_refresh = Instant::now() + TRAY_REFRESH;
        }
        *control_flow = ControlFlow::WaitUntil(next_refresh);
    });
}

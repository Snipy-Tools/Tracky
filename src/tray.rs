use crate::{autostart, focus::Focus, icon};
use std::sync::Arc;
use tao::event_loop::{EventLoop, EventLoopProxy};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const ICON_SIZE: u32 = 64;

pub enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
    OpenDashboard,
    Quit,
}

pub struct Tray {
    icon: TrayIcon,
    open: MenuItem,
    pause: CheckMenuItem,
    autostart: CheckMenuItem,
    quit: MenuItem,
    focus: Arc<Focus>,
    proxy: EventLoopProxy<UserEvent>,
}

fn duration(secs: u64) -> String {
    let m = secs / 60;
    if m < 60 { format!("{m}m") } else { format!("{}h {:02}m", m / 60, m % 60) }
}

impl Tray {
    pub fn new(event_loop: &EventLoop<UserEvent>, focus: Arc<Focus>) -> Self {
        let proxy = event_loop.create_proxy();
        TrayIconEvent::set_event_handler(Some(move |e| {
            let _ = proxy.send_event(UserEvent::Tray(e));
        }));
        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |e| {
            let _ = proxy.send_event(UserEvent::Menu(e));
        }));

        let open = MenuItem::new("Open dashboard", true, None);
        let pause = CheckMenuItem::new("Pause tracking", true, false, None);
        let autostart = CheckMenuItem::new("Start with Windows", true, autostart::is_enabled(), None);
        let quit = MenuItem::new("Quit", true, None);
        let menu = Menu::new();
        menu.append_items(&[&open, &PredefinedMenuItem::separator(), &pause, &autostart, &PredefinedMenuItem::separator(), &quit])
            .expect("tray menu failed to build");

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Tracky")
            .with_icon(Icon::from_rgba(icon::rgba(ICON_SIZE), ICON_SIZE, ICON_SIZE).expect("invalid icon data"))
            .build()
            .expect("tray icon failed to build");

        Self { icon, open, pause, autostart, quit, focus, proxy: event_loop.create_proxy() }
    }

    /// Keeps tooltip and pause checkbox in sync (the dashboard can pause too).
    pub fn refresh(&self) {
        let paused = self.focus.is_paused();
        self.pause.set_checked(paused);
        let today = duration(self.focus.today_seconds());
        let tip = if paused { format!("Tracky · paused · {today} today") } else { format!("Tracky · {today} today") };
        self.icon.set_tooltip(Some(tip)).ok();
    }

    pub fn handle_event(&self, event: UserEvent) {
        match event {
            UserEvent::Tray(TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }) => {
                self.proxy.send_event(UserEvent::OpenDashboard).ok();
            }
            UserEvent::Menu(e) if e.id == self.open.id() => {
                self.proxy.send_event(UserEvent::OpenDashboard).ok();
            }
            UserEvent::Menu(e) if e.id == self.pause.id() => {
                self.focus.set_paused(self.pause.is_checked());
                self.refresh();
            }
            UserEvent::Menu(e) if e.id == self.autostart.id() => {
                let want = self.autostart.is_checked();
                if !autostart::set(want) {
                    self.autostart.set_checked(!want);
                }
            }
            UserEvent::Menu(e) if e.id == self.quit.id() => {
                self.proxy.send_event(UserEvent::Quit).ok();
            }
            _ => {}
        }
    }
}

use crate::focus::Focus;
use crate::icon;
use std::borrow::Cow;
use std::sync::Arc;
use tao::dpi::LogicalSize;
use tao::event::WindowEvent;
use tao::event_loop::EventLoopWindowTarget;
use tao::window::{Icon, Window, WindowBuilder, WindowId};
use wry::http::{Request, Response, header::CONTENT_TYPE};
use wry::{WebContext, WebView, WebViewBuilder};

pub struct Dashboard {
    window: Window,
    webview: WebView,
}

fn respond(ctype: &str, body: impl Into<Cow<'static, [u8]>>) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .header(CONTENT_TYPE, ctype)
        .header("Cache-Control", "no-store")
        .body(body.into())
        .unwrap()
}

fn query<'a>(req: &'a Request<Vec<u8>>, key: &str) -> &'a str {
    req.uri()
        .query()
        .unwrap_or("")
        .split('&')
        .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))
        .unwrap_or("")
}

impl Dashboard {
    pub fn new<T>(target: &EventLoopWindowTarget<T>, context: &mut WebContext, focus: Arc<Focus>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Tracky")
            .with_window_icon(Icon::from_rgba(icon::rgba(64), 64, 64).ok())
            .with_inner_size(LogicalSize::new(1180.0, 820.0))
            .with_min_inner_size(LogicalSize::new(720.0, 560.0))
            .with_visible(false)
            .build(target)
            .expect("dashboard failed to build");

        let html = include_str!("./ui/dashboard.html").replace("{{LOGO}}", &icon::data_uri());

        // Served on Windows as http://tracky.localhost/
        let webview = WebViewBuilder::new_with_web_context(context)
            .with_background_color((17, 25, 23, 255))
            .with_custom_protocol("tracky".into(), move |_, req| {
                let json = "application/json; charset=utf-8";
                match req.uri().path() {
                    "/api/sessions" => respond(json, focus.sessions_json(query(&req, "dates")).into_bytes()),
                    "/api/status" => respond(json, focus.status_json().into_bytes()),
                    "/api/pause" => {
                        focus.set_paused(query(&req, "on") == "true");
                        respond(json, focus.status_json().into_bytes())
                    }
                    _ => respond("text/html; charset=utf-8", html.clone().into_bytes()),
                }
            })
            .with_url("http://tracky.localhost/")
            .build(&window)
            .expect("dashboard webview failed to build");

        Self { window, webview }
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn show(&self) {
        self.webview.evaluate_script("window.refresh && (refresh(), status())").ok();
        self.window.set_visible(true);
        self.window.set_minimized(false);
        self.window.set_focus();
    }

    pub fn handle_event(&self, event: &WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            self.window.set_visible(false);
        }
    }
}

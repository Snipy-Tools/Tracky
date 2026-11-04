//! Focus tracker: records which window is in the foreground while you are actually
//! active. Idle time (no input for IDLE_MS, except video/call windows), the lock
//! screen and Tracky's own windows are not counted.
//!
//! Sessions are appended to `%APPDATA%\Tracky\sessions\YYYY-MM-DD.tsv` as
//! `start_ms \t end_ms \t exe \t title`. Long sessions are written in chunks so a
//! crash loses at most CHUNK_MS.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::SystemInformation::{GetLocalTime, GetTickCount};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
use windows::core::PWSTR;

const TICK: Duration = Duration::from_secs(1);
const IDLE_MS: u32 = 120_000;
const CHUNK_MS: u64 = 60_000;
const MIN_SESSION_MS: u64 = 1_000;

const IGNORED: &[&str] = &[
    "lockapp.exe", "searchhost.exe", "shellexperiencehost.exe", "startmenuexperiencehost.exe",
    "screenclippinghost.exe",
];
const IDLE_EXEMPT: &[&str] = &["youtube", "netflix", "twitch", "vlc", "prime video", "disney+", "zoom", "teams", "meet"];

struct Session {
    date: String,
    start: u64,
    end: u64,
    exe: String,
    title: String,
}

#[derive(Default)]
struct State {
    paused: bool,
    current: Option<Session>,
}

pub struct Focus {
    dir: PathBuf,
    state: Mutex<State>,
}

pub fn start() -> Arc<Focus> {
    let dir = PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".into())).join("Tracky").join("sessions");
    fs::create_dir_all(&dir).ok();
    let focus = Arc::new(Focus { dir, state: Mutex::new(State::default()) });
    let worker = Arc::clone(&focus);
    spawn(move || loop {
        worker.tick();
        sleep(TICK);
    });
    focus
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn local_date() -> String {
    let t = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay)
}

fn idle_ms() -> u32 {
    let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        unsafe { GetTickCount() }.wrapping_sub(info.dwTime)
    } else {
        0
    }
}

/// (exe file name in lowercase, window title) of the focused window.
fn foreground() -> Option<(String, String)> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) }.max(0) as usize;
    let title = String::from_utf16_lossy(&buf[..len]);

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 || pid == std::process::id() {
        return None;
    }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut path = [0u16; 1024];
    let mut size = path.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(path.as_mut_ptr()), &mut size) };
    unsafe { CloseHandle(process) }.ok();
    ok.ok()?;
    let path = String::from_utf16_lossy(&path[..size as usize]);
    let exe = path.rsplit('\\').next()?.to_lowercase();
    Some((exe, clean(&title)))
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ").trim().to_string()
}

impl Focus {
    fn tick(&self) {
        let fg = foreground().filter(|(exe, title)| {
            if IGNORED.contains(&exe.as_str()) {
                return false;
            }
            // desktop, Alt+Tab and taskbar are not work
            if exe == "explorer.exe" && matches!(title.as_str(), "" | "Program Manager" | "Task Switching" | "Task View" | "System tray overflow window.") {
                return false;
            }
            let haystack = format!("{exe} {}", title.to_lowercase());
            idle_ms() < IDLE_MS || IDLE_EXEMPT.iter().any(|k| haystack.contains(k))
        });
        let now = now_ms();
        let mut state = self.state.lock().unwrap();
        let Some((exe, title)) = fg.filter(|_| !state.paused) else {
            if let Some(s) = state.current.take() {
                self.write(&s);
            }
            return;
        };

        if let Some(cur) = state.current.as_mut() {
            // gaps (sleep, hibernate) end a session
            if cur.exe == exe && cur.title == title && now.saturating_sub(cur.end) < 3_000 {
                cur.end = now;
                if cur.end - cur.start >= CHUNK_MS {
                    self.write(cur);
                    cur.start = cur.end;
                    cur.date = local_date();
                }
                return;
            }
        }
        // a direct switch continues where the previous window ended
        let start = match state.current.take() {
            Some(s) => {
                self.write(&s);
                if now.saturating_sub(s.end) < 3_000 { s.end } else { now }
            }
            None => now,
        };
        state.current = Some(Session { date: local_date(), start, end: now, exe, title });
    }

    fn write(&self, s: &Session) {
        if s.end.saturating_sub(s.start) < MIN_SESSION_MS {
            return;
        }
        let path = self.dir.join(format!("{}.tsv", s.date));
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            writeln!(f, "{}\t{}\t{}\t{}", s.start, s.end, s.exe, s.title).ok();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.state.lock().unwrap().paused
    }

    pub fn set_paused(&self, paused: bool) {
        let mut state = self.state.lock().unwrap();
        state.paused = paused;
        if paused {
            if let Some(s) = state.current.take() {
                self.write(&s);
            }
        }
    }

    fn read_day(&self, date: &str) -> Vec<(u64, u64, String, String)> {
        let mut rows: Vec<_> = fs::read_to_string(self.dir.join(format!("{date}.tsv")))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let mut p = line.splitn(4, '\t');
                Some((p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.to_string(), p.next().unwrap_or("").to_string()))
            })
            .collect();
        let state = self.state.lock().unwrap();
        if let Some(s) = state.current.as_ref().filter(|s| s.date == date && s.end > s.start) {
            rows.push((s.start, s.end, s.exe.clone(), s.title.clone()));
        }
        rows
    }

    /// Focused seconds recorded today, for the popup.
    pub fn today_seconds(&self) -> u64 {
        self.read_day(&local_date()).iter().map(|r| r.1 - r.0).sum::<u64>() / 1000
    }

    /// `{"2026-11-04": [[start, end, "exe", "title"], ...], ...}`
    pub fn sessions_json(&self, dates: &str) -> String {
        let days: Vec<String> = dates
            .split(',')
            .filter(|d| d.len() == 10 && d.bytes().all(|b| b.is_ascii_digit() || b == b'-'))
            .take(31)
            .map(|d| {
                let rows: Vec<String> = self
                    .read_day(d)
                    .iter()
                    .map(|(s, e, exe, title)| format!("[{s},{e},{},{}]", json_str(exe), json_str(title)))
                    .collect();
                format!("\"{d}\":[{}]", rows.join(","))
            })
            .collect();
        format!("{{{}}}", days.join(","))
    }

    pub fn status_json(&self) -> String {
        let state = self.state.lock().unwrap();
        match &state.current {
            Some(s) => format!(
                "{{\"paused\":{},\"exe\":{},\"title\":{},\"since\":{}}}",
                state.paused, json_str(&s.exe), json_str(&s.title), s.start
            ),
            None => format!("{{\"paused\":{},\"exe\":null}}", state.paused),
        }
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

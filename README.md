# Tracky

Tracky lives in the Windows tray and records which window you are focused on during the day. A dashboard shows how your day went: what you worked on, when, and for how long.

Part of [Snipy Tools](https://github.com/Snipy-Tools).

## What gets counted

Only the window in the foreground, and only while you are actually there:

- No mouse or keyboard input for 2 minutes stops the clock. Windows with YouTube, Netflix, Twitch, Zoom or Teams in the title keep counting, so watching a video or sitting in a call still counts.
- The lock screen, desktop, Alt+Tab and Tracky's own window are not counted.

## Dashboard

Left-click the tray icon (or start `tracky.exe` by hand) to open it.

- **Focus time**, **deep work** (blocks of 25+ min in Coding, Learning or Design), **longest focus block** and **app switches**
- **Your day**: a timeline coloured by category; hover a segment to see the windows behind it
- **Per hour**: minutes of focus per hour, stacked by category
- **Categories** and **apps**: click an app to see its windows and websites
- **Last 7 days**: click a day to open it
- **Focus blocks**: your longest stretches on one thing

The current window and a pause button are shown at the top. Use ← and → to switch days.

Browser tabs are sorted by their title: GitHub counts as Coding, YouTube as Media, Moodle as Learning, and so on. App names and categories are listed in `APPS` and `WEB_RULES` at the top of the script in `src/ui/dashboard.html`.

## Tray menu

| Item | |
|---|---|
| Open dashboard | same as left-click |
| Pause tracking | stops recording until you resume (resets on restart) |
| Start with Windows | on by default: Tracky turns it on the first time it runs (`HKCU\...\CurrentVersion\Run`) and then starts silently in the tray |
| Quit | saves the running session and exits |

Hovering the tray icon shows today's focus time.

## Data

Everything stays on your PC: `%APPDATA%\Tracky\sessions\YYYY-MM-DD.tsv`, one line per session:

```
start_ms	end_ms	exe	window title
```

Long sessions are saved every minute, so a crash loses at most one minute.

## Build

Requires Rust and Windows (WebView2 ships with Windows 10/11).

```bash
cargo build --release
```

The binary is `target/release/tracky.exe`. Release builds run without a console window.

## Release

Pushing a tag like `v0.1.0` builds `tracky-Windows.exe` and an installer (`tracky-Windows-Setup.exe`, Inno Setup, see `installer.iss`) and attaches both to a GitHub release. The installer has a "Start Tracky with Windows" checkbox.

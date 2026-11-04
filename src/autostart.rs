//! "Start with Windows" via the per-user Run key, like Snipy.

use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::{PCWSTR, w};

const RUN_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const VALUE: PCWSTR = w!("Tracky");
/// Set once Tracky (or its installer) has decided about autostart.
const APP_KEY: PCWSTR = w!(r"Software\Tracky");
const INITIALIZED: PCWSTR = w!("Initialized");

pub fn is_enabled() -> bool {
    let mut key = HKEY::default();
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, KEY_READ, &mut key).is_err() {
            return false;
        }
        let found = RegQueryValueExW(key, VALUE, None, None, None, None).is_ok();
        let _ = RegCloseKey(key);
        found
    }
}

pub fn set(enabled: bool) -> bool {
    let mut key = HKEY::default();
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, KEY_SET_VALUE, &mut key).is_err() {
            return false;
        }
        let ok = if enabled {
            match std::env::current_exe() {
                Ok(exe) => {
                    let command = format!("\"{}\" --background", exe.display());
                    let wide: Vec<u16> = command.encode_utf16().chain(Some(0)).collect();
                    let bytes = std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * 2);
                    RegSetValueExW(key, VALUE, None, REG_SZ, Some(bytes)).is_ok()
                }
                Err(_) => false,
            }
        } else {
            RegDeleteValueW(key, VALUE).is_ok()
        };
        let _ = RegCloseKey(key);
        ok
    }
}

/// A time tracker is only useful if it runs all day, so autostart is switched on
/// the first time Tracky starts. Afterwards the tray menu decides; the installer
/// sets the marker itself so its "Start with Windows" checkbox is respected.
pub fn enable_on_first_run() {
    let mut key = HKEY::default();
    unsafe {
        if RegCreateKeyExW(HKEY_CURRENT_USER, APP_KEY, None, None, REG_OPTION_NON_VOLATILE, KEY_READ | KEY_WRITE, None, &mut key, None)
            .is_err()
        {
            return;
        }
        if RegQueryValueExW(key, INITIALIZED, None, None, None, None).is_err() {
            set(true);
            let one = 1u32.to_le_bytes();
            let _ = RegSetValueExW(key, INITIALIZED, None, REG_DWORD, Some(&one));
        }
        let _ = RegCloseKey(key);
    }
}

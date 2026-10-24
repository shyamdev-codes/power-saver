pub mod canvas;
pub mod theme;

use std::mem::zeroed;
use std::ptr::null;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::app::AppState;
use crate::win32::*;

pub static IS_EXITING: AtomicBool = AtomicBool::new(false);
pub static GLOBAL_APP: Mutex<Option<AppState>> = Mutex::new(None);

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER => {
            if let Ok(mut lock) = GLOBAL_APP.lock() {
                if let Some(app) = lock.as_mut() {
                    app.tick();
                }
            }
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_CHAR => {
            if let Some(c) = char::from_u32(wparam as u32) {
                if let Ok(mut lock) = GLOBAL_APP.lock() {
                    if let Some(app) = lock.as_mut() {
                        app.handle_char(c);
                    }
                }
                unsafe { InvalidateRect(hwnd, null(), 0) };
            }
            0
        }
        WM_KEYDOWN => {
            if let Ok(mut lock) = GLOBAL_APP.lock() {
                if let Some(app) = lock.as_mut() {
                    app.handle_key_down(wparam);
                }
            }
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_LBUTTONUP => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            if let Ok(mut lock) = GLOBAL_APP.lock() {
                if let Some(app) = lock.as_mut() {
                    app.handle_click(x, y);
                }
            }
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_TRAYICON => {
            match lparam as u32 {
                WM_LBUTTONUP => unsafe {
                    ShowWindow(hwnd, SW_SHOW);
                    ShowWindow(hwnd, SW_SHOWNORMAL);
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                },
                WM_RBUTTONUP => unsafe {
                    let mut pt: POINT = zeroed();
                    GetCursorPos(&mut pt);
                    let menu = CreatePopupMenu();
                    let s_open = to_wide("Open Dashboard");
                    let is_auto = crate::persistence::is_autostart_enabled();
                    let auto_label = if is_auto {
                        "[v] Start on Windows Boot (Enabled)"
                    } else {
                        "[ ] Start on Windows Boot (Disabled)"
                    };
                    let s_auto = to_wide(auto_label);
                    let s_exit = to_wide("Exit & Restore 100%");
                    AppendMenuW(menu, MF_STRING, ID_TRAY_OPEN, s_open.as_ptr());
                    AppendMenuW(menu, MF_STRING, ID_TRAY_AUTOSTART, s_auto.as_ptr());
                    AppendMenuW(menu, MF_SEPARATOR, 0, null());
                    AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, s_exit.as_ptr());
                    SetForegroundWindow(hwnd);
                    TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, null());
                    DestroyMenu(menu);
                },
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            match wparam {
                ID_TRAY_OPEN => unsafe {
                    ShowWindow(hwnd, SW_SHOW);
                    ShowWindow(hwnd, SW_SHOWNORMAL);
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                },
                ID_TRAY_AUTOSTART => {
                    let is_auto = crate::persistence::is_autostart_enabled();
                    if let Ok(exe_path) = std::env::current_exe() {
                        let _ = crate::persistence::set_autostart_enabled(!is_auto, &exe_path);
                    }
                }
                ID_TRAY_EXIT => unsafe {
                    IS_EXITING.store(true, Ordering::SeqCst);
                    ShowWindow(hwnd, SW_HIDE);
                    PostQuitMessage(0);
                },
                _ => {}
            }
            0
        }
        WM_CLOSE => unsafe {
            // Minimize to system tray instead of exiting
            ShowWindow(hwnd, SW_HIDE);
            0
        },
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = unsafe { zeroed() };
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
            if let Ok(lock) = GLOBAL_APP.lock() {
                if let Some(app) = lock.as_ref() {
                    unsafe { canvas::render_ui(hdc, app) };
                }
            }
            unsafe { EndPaint(hwnd, &ps) };
            0
        }
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            0
        },
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod governor;
pub mod persistence;
pub mod power;
pub mod telemetry;

#[cfg(target_os = "windows")]
pub mod ui;
#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::ptr::{null, null_mut};

#[cfg(target_os = "windows")]
use app::AppState;
use power::PowerManager;
#[cfg(target_os = "windows")]
use ui::{wnd_proc, GLOBAL_APP};
#[cfg(target_os = "windows")]
use win32::*;

fn handle_cli_args(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--restore" || a == "-Restore") {
        PowerManager::set_limit(100);
        return true;
    }
    if args.iter().any(|a| a == "--autostart" || a == "-Autostart") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe_path);
        }
        return true;
    }
    if args.iter().any(|a| a == "--remove-autostart" || a == "-RemoveAutostart") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe_path);
        }
        return true;
    }
    if args.iter().any(|a| a == "--uninstall" || a == "-Uninstall") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe_path);
        }
        PowerManager::set_limit(100);
        return true;
    }
    false
}

fn ensure_autostart_registered() {
    if !persistence::is_autostart_enabled()
        && let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe_path);
        }
}

fn resolve_root_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    if let Some(parent) = exe_dir.parent()
        && let Some(grand) = parent.parent()
            && grand.join("Cargo.toml").exists() {
                return grand.to_path_buf();
            }
    exe_dir
}

#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if handle_cli_args(&args) {
        return;
    }

    let mutex_name = to_wide("Local\\AdaptivePowerGovernorSingleInstanceMutex");
    let class_name = to_wide("AdaptivePowerGovernorRustClass");
    let _h_mutex = unsafe { CreateMutexW(null_mut(), 0, mutex_name.as_ptr()) };
    if unsafe { GetLastError() } == 183 {
        unsafe {
            let existing_hwnd = FindWindowW(class_name.as_ptr(), null());
            if !existing_hwnd.is_null() && IsWindow(existing_hwnd) != 0 {
                AllowSetForegroundWindow(ASFW_ANY);
                ShowWindow(existing_hwnd, SW_SHOW);
                ShowWindow(existing_hwnd, SW_SHOWNORMAL);
                ShowWindow(existing_hwnd, SW_RESTORE);
                SetForegroundWindow(existing_hwnd);
                PostMessageW(existing_hwnd, WM_COMMAND, ID_TRAY_OPEN, 0);
                return;
            }
        }
    }

    ensure_autostart_registered();

    let is_silent = args.iter().any(|a| a == "--silent" || a == "-Silent");
    let root_dir = resolve_root_dir();

    let app = AppState::new(&root_dir);
    if let Ok(mut lock) = GLOBAL_APP.lock() {
        *lock = Some(app);
    }

    unsafe {
        let h_instance = GetModuleHandleW(null());
        init_dpi_awareness();

        let wc = WNDCLASSEXW {
            cb_size: size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wndproc: Some(wnd_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: LoadIconW(null_mut(), 32512 as *const u16),
            h_cursor: null_mut(),
            hbr_background: null_mut(),
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: null_mut(),
        };

        RegisterClassExW(&wc);

        let client_w = 960;
        let client_h = 520;
        let win_style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: client_w,
            bottom: client_h,
        };
        AdjustWindowRectEx(&mut rc, win_style, 0, 0);
        let win_w = rc.right - rc.left;
        let win_h = rc.bottom - rc.top;

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = ((screen_w - win_w) / 2).max(40);
        let pos_y = ((screen_h - win_h) / 2).max(40);

        let window_title = to_wide("Adaptive Power Governor");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            win_style,
            pos_x,
            pos_y,
            win_w,
            win_h,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        enable_dark_title_bar(hwnd);
        SetTimer(hwnd, 1, 1000, None);

        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cb_size = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hwnd = hwnd;
        nid.u_id = 1;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_INFO;
        nid.u_callback_message = WM_TRAYICON;
        nid.h_icon = LoadIconW(null_mut(), 32512 as *const u16);

        let tip = to_wide("Adaptive Power Governor (Autonomous)");
        for (i, c) in tip.iter().enumerate().take(127) {
            nid.sz_tip[i] = *c;
        }

        let info_title = to_wide("Adaptive Power Governor");
        for (i, c) in info_title.iter().enumerate().take(63) {
            nid.sz_info_title[i] = *c;
        }

        let info_body = to_wide("Running silently in tray, continuing to save power.");
        for (i, c) in info_body.iter().enumerate().take(255) {
            nid.sz_info[i] = *c;
        }

        Shell_NotifyIconW(NIM_ADD, &mut nid);

        if !is_silent {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_SHOWNORMAL);
            UpdateWindow(hwnd);
            SetForegroundWindow(hwnd);
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        KillTimer(hwnd, 1);
        Shell_NotifyIconW(NIM_DELETE, &mut nid);

        PowerManager::set_limit(100);
        if let Ok(mut lock) = GLOBAL_APP.lock()
            && let Some(app) = lock.as_mut() {
                app.flush_csv();
                app.governor.current_state = governor::PowerState::Boost100;
                app.update_journal("Default-100", "Restored-100%");
            }
    }
}

#[cfg(target_os = "linux")]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if handle_cli_args(&args) {
        return;
    }

    ensure_autostart_registered();

    let exe_dir = resolve_root_dir();
    let mut app = AppState::new(&exe_dir);
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");

    println!("Adaptive Power Governor — Linux Daemon");
    println!("System : {}", app.system_name);
    println!("CPU    : {} ({} cores, {} MHz max)", app.cpu_name, app.core_count, app.max_mhz);
    println!("Running autonomously. Press Ctrl+C to exit.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        app.tick();

        if verbose {
            println!(
                "[{:>5}s] Load: {:>5.1}% | {:>5} MHz | {} | Live: {:.1}W | Saved: {:.1}W",
                app.session_seconds,
                app.current_load,
                app.current_mhz,
                app.governor.current_state.profile_name(),
                app.live_watts,
                app.unthrottled_watts - app.live_watts,
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn enable_dark_title_bar(hwnd: HWND) {
    unsafe {
        let h_mod = LoadLibraryA(c"dwmapi.dll".as_ptr() as *const u8);
        if !h_mod.is_null() {
            let p_dwm = GetProcAddress(h_mod, c"DwmSetWindowAttribute".as_ptr() as *const u8);
            if !p_dwm.is_null() {
                let set_attr: unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> i32 =
                    std::mem::transmute::<*mut c_void, _>(p_dwm);
                let dark: i32 = 1;
                let _ = set_attr(hwnd, 20, &dark as *const i32 as *const c_void, 4);
                let _ = set_attr(hwnd, 19, &dark as *const i32 as *const c_void, 4);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn init_dpi_awareness() {
    unsafe {
        let h_user = LoadLibraryA(c"user32.dll".as_ptr() as *const u8);
        if !h_user.is_null() {
            let p_ctx = GetProcAddress(h_user, c"SetProcessDpiAwarenessContext".as_ptr() as *const u8);
            if !p_ctx.is_null() {
                let set_ctx: unsafe extern "system" fn(isize) -> i32 =
                    std::mem::transmute::<*mut c_void, _>(p_ctx);
                let _ = set_ctx(-4);
            } else {
                let p_dpi = GetProcAddress(h_user, c"SetProcessDPIAware".as_ptr() as *const u8);
                if !p_dpi.is_null() {
                    let set_dpi: unsafe extern "system" fn() -> i32 =
                        std::mem::transmute::<*mut c_void, _>(p_dpi);
                    let _ = set_dpi();
                }
            }
        }
    }
}

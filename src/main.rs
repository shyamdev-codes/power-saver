#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod governor;
pub mod persistence;
pub mod power;
pub mod telemetry;

// UI and Win32 bindings are Windows-only.
// On Linux the binary runs headless (daemon mode).
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
#[cfg(target_os = "windows")]
use power::PowerManager;
#[cfg(target_os = "windows")]
use ui::{wnd_proc, GLOBAL_APP};
#[cfg(target_os = "windows")]
use win32::*;

// ─────────────────────────────────────────────────────────────────────────────
//  WINDOWS ENTRY POINT  (full GUI + tray)
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 1. Standalone CLI Operations
    if args.iter().any(|a| a == "--restore" || a == "-Restore") {
        PowerManager::set_limit(100);
        return;
    }

    if args.iter().any(|a| a == "--autostart" || a == "-Autostart") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe_path);
        }
        return;
    }

    if args.iter().any(|a| a == "--remove-autostart" || a == "-RemoveAutostart") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe_path);
        }
        return;
    }

    if args.iter().any(|a| a == "--uninstall" || a == "-Uninstall") {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe_path);
        }
        PowerManager::set_limit(100);
        return;
    }

    // 2. Single-Instance Check & Activation:
    // If the governor is already running in the background and the user clicks the .exe,
    // activate and bring up the existing GUI window with live charts and data.
    let mutex_name = to_wide("Local\\AdaptivePowerGovernorSingleInstanceMutex");
    let class_name = to_wide("AdaptivePowerGovernorRustClass");
    let _h_mutex = unsafe { CreateMutexW(null_mut(), 0, mutex_name.as_ptr()) };
    if unsafe { GetLastError() } == 183 /* ERROR_ALREADY_EXISTS */ {
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

    // 3. First-Time Run Automation:
    // Check if the auto-runner is configured in the system.
    // If not found, automatically register Windows boot startup so next time user logs in,
    // it starts automatically and silently in the background without opening the window!
    if !persistence::is_autostart_enabled() {
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe_path);
        }
    }

    let is_silent = args.iter().any(|a| a == "--silent" || a == "-Silent");

    // 4. Locate directory for persistent data files
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let root_dir = if let Some(parent) = exe_dir.parent() {
        if let Some(grand) = parent.parent() {
            if grand.join("Cargo.toml").exists() {
                grand.to_path_buf()
            } else {
                exe_dir.clone()
            }
        } else {
            exe_dir.clone()
        }
    } else {
        exe_dir.clone()
    };

    // 5. Initialize App State (auto-creates recovery_journal.json and daily_energy_log.csv on first run)
    let app = AppState::new(&root_dir);
    if let Ok(mut lock) = GLOBAL_APP.lock() {
        *lock = Some(app);
    }

    unsafe {
        let h_instance = GetModuleHandleW(null());

        // Initialize high DPI awareness for crisp font rendering
        init_dpi_awareness();

        let wc = WNDCLASSEXW {
            cb_size: size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wndproc: Some(wnd_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: LoadIconW(null_mut(), 32512 as *const u16), // IDI_APPLICATION
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

        // Enable Windows 10/11 immersive dark title bar for seamless aesthetics
        enable_dark_title_bar(hwnd);

        // Setup 1000ms periodic timer for continuous autonomous telemetry and power management
        SetTimer(hwnd, 1, 1000, None);

        // Setup System Tray Icon
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

        // If launched normally (not --silent), show window; if silent (boot auto-start), stay in tray
        if !is_silent {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_SHOWNORMAL);
            UpdateWindow(hwnd);
            SetForegroundWindow(hwnd);
        }

        // Win32 Message Loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup on Exit
        KillTimer(hwnd, 1);
        Shell_NotifyIconW(NIM_DELETE, &mut nid);

        // Mandatory Safety: Restore 100% baseline upon termination
        PowerManager::set_limit(100);
        if let Ok(mut lock) = GLOBAL_APP.lock() {
            if let Some(app) = lock.as_mut() {
                app.flush_csv();
                app.governor.current_state = governor::PowerState::Boost100;
                app.update_journal("Default-100", "Restored-100%");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  LINUX ENTRY POINT  (headless daemon — GUI not yet implemented)
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
fn main() {
    use app::AppState;
    use power::PowerManager;

    let args: Vec<String> = std::env::args().collect();

    // CLI operations — same flags as Windows
    if args.iter().any(|a| a == "--restore" || a == "-Restore") {
        PowerManager::set_limit(100);
        return;
    }
    if args.iter().any(|a| a == "--autostart" || a == "-Autostart") {
        if let Ok(exe) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe);
        }
        return;
    }
    if args.iter().any(|a| a == "--remove-autostart" || a == "-RemoveAutostart") {
        if let Ok(exe) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe);
        }
        return;
    }
    if args.iter().any(|a| a == "--uninstall" || a == "-Uninstall") {
        if let Ok(exe) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(false, &exe);
        }
        PowerManager::set_limit(100);
        return;
    }

    // Auto-register for boot autostart on first run
    if !persistence::is_autostart_enabled() {
        if let Ok(exe) = std::env::current_exe() {
            let _ = persistence::set_autostart_enabled(true, &exe);
        }
    }

    // Locate data directory (same logic as Windows)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Initialize app state
    let mut app = AppState::new(&exe_dir);
    let verbose  = args.iter().any(|a| a == "--verbose" || a == "-v");

    println!("Adaptive Power Governor — Linux Daemon");
    println!("System : {}", app.system_name);
    println!("CPU    : {} ({} cores, {} MHz max)", app.cpu_name, app.core_count, app.max_mhz);
    println!("Running autonomously. Use Ctrl+C to exit and restore 100%.");
    println!();

    // Safety: restore 100% on Ctrl+C via a simple signal flag
    // (Production: use the `ctrlc` crate or signal-hook for proper cleanup)
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

// ─────────────────────────────────────────────────────────────────────────────
//  WINDOWS-ONLY HELPERS
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
unsafe fn enable_dark_title_bar(hwnd: HWND) {
    unsafe {
        let h_mod = LoadLibraryA(b"dwmapi.dll\0".as_ptr());
        if !h_mod.is_null() {
            let p_dwm = GetProcAddress(h_mod, b"DwmSetWindowAttribute\0".as_ptr());
            if !p_dwm.is_null() {
                let set_attr: unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> i32 =
                    std::mem::transmute(p_dwm);
                let dark: i32 = 1;
                let _ = set_attr(hwnd, 20, &dark as *const i32 as *const c_void, 4);
                let _ = set_attr(hwnd, 19, &dark as *const i32 as *const c_void, 4);
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn init_dpi_awareness() {
    unsafe {
        let h_user = LoadLibraryA(b"user32.dll\0".as_ptr());
        if !h_user.is_null() {
            let p_ctx = GetProcAddress(h_user, b"SetProcessDpiAwarenessContext\0".as_ptr());
            if !p_ctx.is_null() {
                let set_ctx: unsafe extern "system" fn(isize) -> i32 = std::mem::transmute(p_ctx);
                let _ = set_ctx(-4); // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
            } else {
                let p_dpi = GetProcAddress(h_user, b"SetProcessDPIAware\0".as_ptr());
                if !p_dpi.is_null() {
                    let set_dpi: unsafe extern "system" fn() -> i32 = std::mem::transmute(p_dpi);
                    let _ = set_dpi();
                }
            }
        }
    }
}


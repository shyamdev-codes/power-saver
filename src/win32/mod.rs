pub mod types;
pub use types::*;

use std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn LoadLibraryA(lp_lib_file_name: *const u8) -> *mut c_void;
    pub fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const u8) -> *mut c_void;
    pub fn LocalFree(h_mem: *mut c_void) -> *mut c_void;
    pub fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
    pub fn GetSystemTimes(
        lp_idle_time: *mut FILETIME,
        lp_kernel_time: *mut FILETIME,
        lp_user_time: *mut FILETIME,
    ) -> i32;
    pub fn GetSystemTime(lp_system_time: *mut SYSTEMTIME);
    pub fn GetLocalTime(lp_system_time: *mut SYSTEMTIME);
    pub fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut c_void;
    pub fn Process32FirstW(h_snapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
    pub fn Process32NextW(h_snapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
    pub fn CloseHandle(h_object: *mut c_void) -> i32;
    pub fn GetSystemInfo(lp_system_info: *mut SYSTEM_INFO);
    pub fn CreateMutexW(lp_mutex_attributes: *mut c_void, b_initial_owner: i32, lp_name: *const u16) -> *mut c_void;
    pub fn GetLastError() -> u32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn RegOpenKeyExW(
        h_key: usize,
        lp_sub_key: *const u16,
        ul_options: u32,
        sam_desired: u32,
        phk_result: *mut usize,
    ) -> i32;
    pub fn RegQueryValueExW(
        h_key: usize,
        lp_value_name: *const u16,
        lp_reserved: *mut u32,
        lp_type: *mut u32,
        lp_data: *mut u8,
        lpcb_data: *mut u32,
    ) -> i32;
    pub fn RegSetValueExW(
        h_key: usize,
        lp_value_name: *const u16,
        reserved: u32,
        dw_type: u32,
        lp_data: *const u8,
        cb_data: u32,
    ) -> i32;
    pub fn RegDeleteValueW(
        h_key: usize,
        lp_value_name: *const u16,
    ) -> i32;
    pub fn RegCloseKey(h_key: usize) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn AdjustWindowRectEx(lp_rect: *mut RECT, dw_style: u32, b_menu: i32, dw_ex_style: u32) -> i32;
    pub fn GetSystemMetrics(n_index: i32) -> i32;
    pub fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> HWND;
    pub fn RegisterClassExW(lp_wnd_class: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: HWND,
        h_menu: HMENU,
        h_instance: HINSTANCE,
        lp_param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    pub fn IsWindow(h_wnd: HWND) -> i32;
    pub fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> i32;
    pub fn UpdateWindow(h_wnd: HWND) -> i32;
    pub fn GetMessageW(lp_msg: *mut MSG, h_wnd: HWND, w_msg_filter_min: u32, w_msg_filter_max: u32) -> i32;
    pub fn TranslateMessage(lp_msg: *const MSG) -> i32;
    pub fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(n_exit_code: i32);
    pub fn SetTimer(h_wnd: HWND, n_id_event: usize, u_elapse: u32, lp_timer_func: Option<unsafe extern "system" fn(HWND, u32, usize, u32)>) -> usize;
    pub fn KillTimer(h_wnd: HWND, u_id_event: usize) -> i32;
    pub fn BeginPaint(h_wnd: HWND, lp_paint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(h_wnd: HWND, lp_paint: *const PAINTSTRUCT) -> i32;
    pub fn InvalidateRect(h_wnd: HWND, lp_rect: *const RECT, b_erase: i32) -> i32;
    pub fn FillRect(h_dc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub fn DrawTextW(h_dc: HDC, lp_ch_text: *const u16, cch_text: i32, lprc: *mut RECT, format: u32) -> i32;
    pub fn SetBkMode(h_dc: HDC, mode: i32) -> i32;
    pub fn SetTextColor(h_dc: HDC, color: u32) -> u32;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(h_menu: HMENU, u_flags: u32, u_id_new_item: usize, lp_new_item: *const u16) -> i32;
    pub fn DestroyMenu(h_menu: HMENU) -> i32;
    pub fn TrackPopupMenu(h_menu: HMENU, u_flags: u32, x: i32, y: i32, n_reserved: i32, h_wnd: HWND, prc_rect: *const RECT) -> i32;
    pub fn GetCursorPos(lp_point: *mut POINT) -> i32;
    pub fn SetForegroundWindow(h_wnd: HWND) -> i32;
    pub fn AllowSetForegroundWindow(dw_process_id: u32) -> i32;
    pub fn PostMessageW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> i32;
    pub fn LoadIconW(h_instance: HINSTANCE, lp_icon_name: *const u16) -> HICON;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn CreatePen(i_style: i32, c_width: i32, color: u32) -> HPEN;
    pub fn SelectObject(h_dc: HDC, h: *mut c_void) -> *mut c_void;
    pub fn DeleteObject(ho: *mut c_void) -> i32;
    pub fn MoveToEx(h_dc: HDC, x: i32, y: i32, lppt: *mut POINT) -> i32;
    pub fn LineTo(h_dc: HDC, x: i32, y: i32) -> i32;
    pub fn RoundRect(h_dc: HDC, left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> i32;
    pub fn Polygon(h_dc: HDC, apt: *const POINT, cpt: i32) -> i32;
    pub fn CreateCompatibleDC(h_dc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(h_dc: HDC, cx: i32, cy: i32) -> HBITMAP;
    pub fn BitBlt(h_dc: HDC, x: i32, y: i32, cx: i32, cy: i32, hdc_src: HDC, x1: i32, y1: i32, rop: u32) -> i32;
    pub fn DeleteDC(h_dc: HDC) -> i32;
    pub fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> HFONT;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn Shell_NotifyIconW(dw_message: u32, lp_data: *mut NOTIFYICONDATAW) -> i32;
}

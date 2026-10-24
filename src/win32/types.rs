use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;

// =========================================================================
// Win32 Types and Constants
// =========================================================================
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl GUID {
    pub const fn new(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Self {
        Self { data1: d1, data2: d2, data3: d3, data4: d4 }
    }

    pub fn to_string_guid(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

pub const GUID_BALANCED: GUID = GUID::new(
    0x381b4222,
    0xf694,
    0x41f0,
    [0x96, 0x85, 0xff, 0x5b, 0xb2, 0x60, 0xdf, 0x2e],
);
pub const GUID_SUB_PROCESSOR: GUID = GUID::new(
    0x54533251,
    0x82be,
    0x4824,
    [0x96, 0xc1, 0x47, 0xb6, 0x0b, 0x74, 0x0d, 0x00],
);
pub const GUID_PROCTHROTTLEMAX: GUID = GUID::new(
    0xbc5038f7,
    0x23e0,
    0x4960,
    [0x96, 0xda, 0x33, 0xab, 0xaf, 0x59, 0x35, 0xec],
);

pub type HWND = *mut c_void;
pub type HDC = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HPEN = *mut c_void;
pub type HFONT = *mut c_void;
pub type HICON = *mut c_void;
pub type HMENU = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HBITMAP = *mut c_void;
pub type LPARAM = isize;
pub type WPARAM = usize;
pub type LRESULT = isize;
pub type WNDPROC = Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wndproc: WNDPROC,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HINSTANCE,
    pub h_icon: HICON,
    pub h_cursor: *mut c_void,
    pub hbr_background: HBRUSH,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: HICON,
}

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub f_erase: i32,
    pub rc_paint: RECT,
    pub f_restore: i32,
    pub f_inc_update: i32,
    pub rgb_reserved: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FILETIME {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

impl FILETIME {
    pub fn to_u64(&self) -> u64 {
        ((self.dw_high_date_time as u64) << 32) | (self.dw_low_date_time as u64)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SYSTEMTIME {
    pub w_year: u16,
    pub w_month: u16,
    pub w_day_of_week: u16,
    pub w_day: u16,
    pub w_hour: u16,
    pub w_minute: u16,
    pub w_second: u16,
    pub w_milliseconds: u16,
}

#[repr(C)]
pub struct PROCESSENTRY32W {
    pub dw_size: u32,
    pub cnt_usage: u32,
    pub th32_process_id: u32,
    pub th32_default_heap_id: usize,
    pub th32_module_id: u32,
    pub cnt_threads: u32,
    pub th32_parent_process_id: u32,
    pub pc_pri_class_base: i32,
    pub dw_flags: u32,
    pub sz_exe_file: [u16; 260],
}

#[repr(C)]
pub struct SYSTEM_INFO {
    pub w_processor_architecture: u16,
    pub w_reserved: u16,
    pub dw_page_size: u32,
    pub lp_minimum_application_address: *mut c_void,
    pub lp_maximum_application_address: *mut c_void,
    pub dw_active_processor_mask: usize,
    pub dw_number_of_processors: u32,
    pub dw_processor_type: u32,
    pub dw_allocation_granularity: u32,
    pub w_processor_level: u16,
    pub w_processor_revision: u16,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cb_size: u32,
    pub hwnd: HWND,
    pub u_id: u32,
    pub u_flags: u32,
    pub u_callback_message: u32,
    pub h_icon: HICON,
    pub sz_tip: [u16; 128],
    pub dw_state: u32,
    pub dw_state_mask: u32,
    pub sz_info: [u16; 256],
    pub u_timeout_or_version: u32,
    pub sz_info_title: [u16; 64],
    pub dw_info_flags: u32,
    pub guid_item: GUID,
    pub h_balloon_icon: HICON,
}

pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_USER: u32 = 0x0400;
pub const WM_TRAYICON: u32 = WM_USER + 100;

pub const WS_OVERLAPPED: u32 = 0x00000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOW: i32 = 5;
pub const SW_RESTORE: i32 = 9;

pub const DT_LEFT: u32 = 0x00000000;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_RIGHT: u32 = 0x00000002;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_WORDBREAK: u32 = 0x00000010;
pub const DT_SINGLELINE: u32 = 0x00000020;

pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

pub const TRANSPARENT: i32 = 1;
pub const SRCCOPY: u32 = 0x00CC0020;
pub const PS_SOLID: i32 = 0;
pub const PS_DASH: i32 = 1;

pub const NIF_MESSAGE: u32 = 0x00000001;
pub const NIF_ICON: u32 = 0x00000002;
pub const NIF_TIP: u32 = 0x00000004;
pub const NIF_INFO: u32 = 0x00000010;

pub const NIM_ADD: u32 = 0x00000000;
pub const NIM_DELETE: u32 = 0x00000002;

pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_RIGHTALIGN: u32 = 0x0008;
pub const TPM_BOTTOMALIGN: u32 = 0x0020;
pub const MF_STRING: u32 = 0x0000;
pub const MF_SEPARATOR: u32 = 0x0800;

pub const REG_SZ: u32 = 1;
pub const HKEY_CURRENT_USER: usize = 0x80000001;
pub const HKEY_LOCAL_MACHINE: usize = 0x80000002;
pub const KEY_READ: u32 = 0x20019;
pub const KEY_WRITE: u32 = 0x20006;

pub const ID_TRAY_OPEN: usize = 1001;
pub const ID_TRAY_AUTOSTART: usize = 1002;
pub const ID_TRAY_EXIT: usize = 1003;
pub const ASFW_ANY: u32 = 0xFFFFFFFF;

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub struct PowerManager;

#[cfg(target_os = "windows")]
mod win_impl {
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use crate::win32::*;

    pub type FnGetActive = unsafe extern "system" fn(*mut c_void, *mut *mut GUID) -> u32;
    pub type FnWriteAC = unsafe extern "system" fn(*mut c_void, *const GUID, *const GUID, *const GUID, u32) -> u32;
    pub type FnWriteDC = unsafe extern "system" fn(*mut c_void, *const GUID, *const GUID, *const GUID, u32) -> u32;
    pub type FnSetActive = unsafe extern "system" fn(*mut c_void, *const GUID) -> u32;
    pub type FnNtPower = unsafe extern "system" fn(u32, *const c_void, u32, *mut c_void, u32) -> i32;

    pub struct Procs {
        pub get_active: FnGetActive,
        pub write_ac: FnWriteAC,
        pub write_dc: Option<FnWriteDC>,
        pub set_active: FnSetActive,
        pub nt_power: Option<FnNtPower>,
    }
    unsafe impl Send for Procs {}
    unsafe impl Sync for Procs {}

    static PROCS: OnceLock<Option<Procs>> = OnceLock::new();

    pub fn get() -> Option<&'static Procs> {
        PROCS.get_or_init(|| unsafe {
            let lib = LoadLibraryA(c"powrprof.dll".as_ptr() as *const u8);
            if lib.is_null() {
                return None;
            }

            let p_get = GetProcAddress(lib, c"PowerGetActiveScheme".as_ptr() as *const u8);
            let p_wac = GetProcAddress(lib, c"PowerWriteACValueIndex".as_ptr() as *const u8);
            let p_wdc = GetProcAddress(lib, c"PowerWriteDCValueIndex".as_ptr() as *const u8);
            let p_set = GetProcAddress(lib, c"PowerSetActiveScheme".as_ptr() as *const u8);
            let p_nt = GetProcAddress(lib, c"CallNtPowerInformation".as_ptr() as *const u8);

            if p_get.is_null() || p_wac.is_null() || p_set.is_null() {
                return None;
            }
            Some(Procs {
                get_active: std::mem::transmute::<*mut c_void, FnGetActive>(p_get),
                write_ac: std::mem::transmute::<*mut c_void, FnWriteAC>(p_wac),
                write_dc: (!p_wdc.is_null()).then(|| std::mem::transmute::<*mut c_void, FnWriteDC>(p_wdc)),
                set_active: std::mem::transmute::<*mut c_void, FnSetActive>(p_set),
                nt_power: (!p_nt.is_null()).then(|| std::mem::transmute::<*mut c_void, FnNtPower>(p_nt)),
            })
        }).as_ref()
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct ProcessorPowerInfo {
    number: u32,
    max_mhz: u32,
    current_mhz: u32,
    mhz_limit: u32,
    max_idle_state: u32,
    current_idle_state: u32,
}

#[cfg(target_os = "windows")]
impl PowerManager {
    #[must_use]
    pub fn get_active_scheme() -> crate::win32::GUID {
        use std::ptr::null_mut;
        use crate::win32::*;
        unsafe {
            if let Some(p) = win_impl::get() {
                let mut ptr: *mut GUID = null_mut();
                if (p.get_active)(null_mut(), &mut ptr) == 0 && !ptr.is_null() {
                    let guid = *ptr;
                    LocalFree(ptr as *mut std::ffi::c_void);
                    return guid;
                }
            }
            GUID_BALANCED
        }
    }

    pub fn set_limit(limit: u32) -> bool {
        use std::ptr::null_mut;
        use crate::win32::*;
        unsafe {
            if let Some(p) = win_impl::get() {
                let scheme = Self::get_active_scheme();

                let r1 = (p.write_ac)(
                    null_mut(),
                    &scheme,
                    &GUID_SUB_PROCESSOR,
                    &GUID_PROCTHROTTLEMAX,
                    limit,
                );
                if let Some(wdc) = p.write_dc {
                    let _ = wdc(
                        null_mut(),
                        &scheme,
                        &GUID_SUB_PROCESSOR,
                        &GUID_PROCTHROTTLEMAX,
                        limit,
                    );
                }
                let r2 = (p.set_active)(null_mut(), &scheme);
                r1 == 0 && r2 == 0
            } else {
                false
            }
        }
    }

    pub fn get_live_processor_mhz(core_count: u32) -> Option<(u32, u32)> {
        use std::ffi::c_void;
        use std::mem::size_of;

        let p = win_impl::get()?;
        let nt = p.nt_power?;
        let n = core_count.max(1) as usize;
        let mut buf: Vec<ProcessorPowerInfo> = vec![Default::default(); n];
        let sz = (n * size_of::<ProcessorPowerInfo>()) as u32;

        unsafe {
            if nt(11, std::ptr::null(), 0, buf.as_mut_ptr() as *mut c_void, sz) >= 0 {
                let (mut cs, mut ms, mut vc) = (0u64, 0u64, 0u64);
                for c in &buf {
                    if c.current_mhz > 0 {
                        cs += u64::from(c.current_mhz);
                        ms += u64::from(c.max_mhz);
                        vc += 1;
                    }
                }
                if let (Some(avg_cur), Some(avg_max)) = (cs.checked_div(vc), ms.checked_div(vc)) {
                    return Some((avg_cur as u32, avg_max as u32));
                }
            }
        }
        None
    }
}

#[cfg(target_os = "linux")]
impl PowerManager {
    const SYS_CPU: &'static str = "/sys/devices/system/cpu";

    fn core_paths() -> Vec<std::path::PathBuf> {
        std::fs::read_dir(Self::SYS_CPU)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy();
                s.len() > 3 && s.starts_with("cpu") && s[3..].chars().all(|c| c.is_ascii_digit())
            })
            .map(|e| e.path())
            .collect()
    }

    fn max_freq_khz() -> u32 {
        std::fs::read_to_string(format!("{}/cpu0/cpufreq/cpuinfo_max_freq", Self::SYS_CPU))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
    }

    pub fn set_limit(limit: u32) -> bool {
        let max_khz = Self::max_freq_khz();
        if max_khz == 0 {
            return false;
        }
        let target_khz = (u64::from(max_khz) * u64::from(limit) / 100) as u32;
        let mut ok = false;
        for core in Self::core_paths() {
            let path = core.join("cpufreq/scaling_max_freq");
            if std::fs::write(&path, target_khz.to_string()).is_ok() {
                ok = true;
            }
        }
        ok
    }

    pub fn get_live_processor_mhz(_core_count: u32) -> Option<(u32, u32)> {
        let max_khz = Self::max_freq_khz();
        if max_khz == 0 {
            return None;
        }
        let mut sum = 0u64;
        let mut n = 0u64;
        for core in Self::core_paths() {
            if let Ok(v) = std::fs::read_to_string(core.join("cpufreq/scaling_cur_freq")) {
                if let Ok(khz) = v.trim().parse::<u64>() {
                    sum += khz;
                    n += 1;
                }
            }
        }
        if n > 0 {
            Some(((sum / n / 1000) as u32, max_khz / 1000))
        } else {
            None
        }
    }
}

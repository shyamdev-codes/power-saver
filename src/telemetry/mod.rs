pub struct HardwareTelemetry {
    prev_idle: u64,
    prev_kernel: u64,
    prev_user: u64,
}

impl Default for HardwareTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl HardwareTelemetry {
    #[must_use]
    pub fn new() -> Self {
        let mut t = Self {
            prev_idle: 0,
            prev_kernel: 0,
            prev_user: 0,
        };
        t.init_times();
        t
    }

    pub fn read_cpu_info() -> (String, u32) {
        use std::ptr::null_mut;
        use crate::win32::*;

        let mut name = String::new();
        let mut mhz = 0u32;

        unsafe {
            let sub_key = to_wide("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
            let mut h_key: usize = 0;
            if RegOpenKeyExW(0x80000002, sub_key.as_ptr(), 0, 0x20019, &mut h_key) == 0 {
                let mhz_name = to_wide("~MHz");
                let mut mhz_val = 0u32;
                let mut size = 4u32;
                if RegQueryValueExW(
                    h_key,
                    mhz_name.as_ptr(),
                    null_mut(),
                    null_mut(),
                    &mut mhz_val as *mut u32 as *mut u8,
                    &mut size,
                ) == 0 && mhz_val > 0
                {
                    mhz = mhz_val;
                }

                let name_key = to_wide("ProcessorNameString");
                let mut buf = [0u16; 128];
                let mut buf_size = (buf.len() * 2) as u32;
                if RegQueryValueExW(
                    h_key,
                    name_key.as_ptr(),
                    null_mut(),
                    null_mut(),
                    buf.as_mut_ptr() as *mut u8,
                    &mut buf_size,
                ) == 0
                {
                    let len = (buf_size / 2) as usize;
                    if len > 0 {
                        let parsed = String::from_utf16_lossy(&buf[..len - 1]);
                        let trimmed = parsed.trim().to_string();
                        if !trimmed.is_empty() {
                            name = trimmed;
                        }
                    }
                }
                RegCloseKey(h_key);
            }
        }

        if mhz == 0 {
            let cores = Self::read_core_count();
            if let Some((_, max_hw)) = crate::power::PowerManager::get_live_processor_mhz(cores) {
                mhz = max_hw;
            }
        }

        if name.is_empty() {
            use crate::win32::*;
            let mut sys: SYSTEM_INFO = unsafe { std::mem::zeroed() };
            unsafe { GetSystemInfo(&mut sys) };
            let arch = match sys.w_processor_architecture {
                9 => "x86_64",
                12 => "ARM64",
                0 => "x86",
                _ => "Unknown",
            };
            name = format!("{arch} Processor ({} Cores)", sys.dw_number_of_processors);
        }

        (name, mhz)
    }

    pub fn read_system_model() -> String {
        use std::ptr::null_mut;
        use crate::win32::*;
        unsafe {
            let sub_key = to_wide("HARDWARE\\DESCRIPTION\\System\\BIOS");
            let mut h_key: usize = 0;
            if RegOpenKeyExW(0x80000002, sub_key.as_ptr(), 0, 0x20019, &mut h_key) == 0 {
                let mut mfg = String::new();
                let mut prod = String::new();

                let mk = to_wide("SystemManufacturer");
                let mut bm = [0u16; 128];
                let mut sm = (bm.len() * 2) as u32;
                if RegQueryValueExW(
                    h_key,
                    mk.as_ptr(),
                    null_mut(),
                    null_mut(),
                    bm.as_mut_ptr() as *mut u8,
                    &mut sm,
                ) == 0 && (sm / 2) as usize > 1
                {
                    mfg = String::from_utf16_lossy(&bm[..(sm / 2) as usize - 1])
                        .trim()
                        .to_string();
                }

                let pk = to_wide("SystemProductName");
                let mut bp = [0u16; 128];
                let mut sp = (bp.len() * 2) as u32;
                if RegQueryValueExW(
                    h_key,
                    pk.as_ptr(),
                    null_mut(),
                    null_mut(),
                    bp.as_mut_ptr() as *mut u8,
                    &mut sp,
                ) == 0 && (sp / 2) as usize > 1
                {
                    prod = String::from_utf16_lossy(&bp[..(sp / 2) as usize - 1])
                        .trim()
                        .to_string();
                }

                RegCloseKey(h_key);
                return match (mfg.is_empty(), prod.is_empty()) {
                    (false, false) => format!("{mfg} {prod}"),
                    (true, false) => prod,
                    (false, true) => mfg,
                    _ => "Windows PC".to_string(),
                };
            }
            "Windows PC".to_string()
        }
    }

    pub fn read_core_count() -> u32 {
        use crate::win32::*;
        unsafe {
            let mut sys: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut sys);
            sys.dw_number_of_processors.max(1)
        }
    }

    pub fn count_processes() -> u32 {
        use std::mem::{size_of, zeroed};
        use crate::win32::*;
        unsafe {
            let snap = CreateToolhelp32Snapshot(0x00000002, 0);
            if (snap as isize) == -1 || snap.is_null() {
                return 0;
            }
            let mut entry: PROCESSENTRY32W = zeroed();
            entry.dw_size = size_of::<PROCESSENTRY32W>() as u32;
            let mut count = 0u32;
            if Process32FirstW(snap, &mut entry) != 0 {
                loop {
                    count += 1;
                    if Process32NextW(snap, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap);
            count
        }
    }

    fn init_times(&mut self) {
        use crate::win32::*;
        unsafe {
            let mut idle = std::mem::zeroed();
            let mut kernel = std::mem::zeroed();
            let mut user = std::mem::zeroed();
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
                self.prev_idle = idle.to_u64();
                self.prev_kernel = kernel.to_u64();
                self.prev_user = user.to_u64();
            }
        }
    }

    pub fn sample_cpu_load(&mut self) -> f64 {
        use crate::win32::*;
        unsafe {
            let mut idle = std::mem::zeroed();
            let mut kernel = std::mem::zeroed();
            let mut user = std::mem::zeroed();
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
                let ci = idle.to_u64();
                let ck = kernel.to_u64();
                let cu = user.to_u64();

                let di = ci.saturating_sub(self.prev_idle);
                let dk = ck.saturating_sub(self.prev_kernel);
                let du = cu.saturating_sub(self.prev_user);

                self.prev_idle = ci;
                self.prev_kernel = ck;
                self.prev_user = cu;

                let total = dk + du;
                if total > 0 {
                    return ((total.saturating_sub(di)) as f64 / total as f64 * 100.0)
                        .clamp(0.0, 100.0);
                }
            }
        }
        0.0
    }
}

#[cfg(target_os = "linux")]
impl HardwareTelemetry {
    #[must_use]
    pub fn new() -> Self {
        let mut t = Self {
            prev_idle: 0,
            prev_kernel: 0,
            prev_user: 0,
        };
        if let Some((idle, total)) = Self::read_proc_stat() {
            t.prev_idle = idle;
            t.prev_kernel = total;
        }
        t
    }

    pub fn read_cpu_info() -> (String, u32) {
        let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let mut name = String::new();
        let mut mhz = 0u32;
        for line in content.lines() {
            if line.starts_with("model name") && name.is_empty() {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    name = val.trim().to_string();
                }
            }
            if (line.starts_with("cpu MHz") || line.starts_with("clock")) && mhz == 0 {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    if let Ok(v) = val.trim().parse::<f64>() {
                        mhz = v.round() as u32;
                    }
                }
            }
        }
        if name.is_empty() {
            name = "Linux Processor".to_string();
        }
        (name, mhz)
    }

    pub fn read_system_model() -> String {
        let mfg = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
            .unwrap_or_default()
            .trim()
            .to_string();
        let prod = std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .unwrap_or_default()
            .trim()
            .to_string();

        match (mfg.is_empty(), prod.is_empty()) {
            (false, false) => format!("{mfg} {prod}"),
            (true, false) => prod,
            (false, true) => mfg,
            _ => "Linux PC".to_string(),
        }
    }

    pub fn read_core_count() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }

    pub fn count_processes() -> u32 {
        std::fs::read_dir("/proc")
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().chars().all(|c| c.is_ascii_digit()))
            .count() as u32
    }

    fn read_proc_stat() -> Option<(u64, u64)> {
        let content = std::fs::read_to_string("/proc/stat").ok()?;
        let first_line = content.lines().next()?;
        if !first_line.starts_with("cpu ") {
            return None;
        }
        let values: Vec<u64> = first_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        if values.len() >= 4 {
            let idle = values[3] + values.get(4).copied().unwrap_or(0);
            let total: u64 = values.iter().sum();
            Some((idle, total))
        } else {
            None
        }
    }

    pub fn sample_cpu_load(&mut self) -> f64 {
        if let Some((idle, total)) = Self::read_proc_stat() {
            let di = idle.saturating_sub(self.prev_idle);
            let dt = total.saturating_sub(self.prev_kernel);

            self.prev_idle = idle;
            self.prev_kernel = total;

            if dt > 0 {
                return (dt.saturating_sub(di) as f64 / dt as f64 * 100.0).clamp(0.0, 100.0);
            }
        }
        0.0
    }
}

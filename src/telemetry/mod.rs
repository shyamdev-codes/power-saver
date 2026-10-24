// ─────────────────────────────────────────────────────────────────────────────
// Hardware Telemetry — Cross-platform
//
//   Windows : GetSystemTimes, Registry (HKLM\HARDWARE\...), Toolhelp32 snapshots
//   Linux   : /proc/stat, /proc/cpuinfo, /sys/class/dmi/id/*, /proc directory
//
// The HardwareTelemetry struct and its field layout are shared.
// Only the method implementations differ per OS.
// ─────────────────────────────────────────────────────────────────────────────

pub struct HardwareTelemetry {
    /// Previous idle time counter (Windows: FILETIME units; Linux: jiffies)
    prev_idle:   u64,
    /// Previous total time counter (Windows: kernel ticks; Linux: total jiffies)
    prev_kernel: u64,
    /// Previous user time counter (Windows: user ticks; Linux: unused — kept for ABI compat)
    prev_user:   u64,
}

// ═════════════════════════════════════════════════════════════════════════════
//  WINDOWS IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(target_os = "windows")]
impl HardwareTelemetry {
    pub fn new() -> Self {
        let mut t = Self { prev_idle: 0, prev_kernel: 0, prev_user: 0 };
        t.init_times();
        t
    }

    /// Discovers CPU model name and rated base frequency from the Windows Registry.
    pub fn read_cpu_info() -> (String, u32) {
        use std::ptr::null_mut;
        use crate::win32::*;

        let mut name = String::new();
        let mut mhz  = 0u32;

        unsafe {
            let sub_key = to_wide("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
            let mut h_key: usize = 0;
            if RegOpenKeyExW(0x80000002, sub_key.as_ptr(), 0, 0x20019, &mut h_key) == 0 {
                // ~MHz
                let mhz_name = to_wide("~MHz");
                let mut mhz_val = 0u32;
                let mut size = 4u32;
                if RegQueryValueExW(h_key, mhz_name.as_ptr(), null_mut(), null_mut(),
                    &mut mhz_val as *mut u32 as *mut u8, &mut size) == 0 && mhz_val > 0
                {
                    mhz = mhz_val;
                }
                // ProcessorNameString
                let name_key = to_wide("ProcessorNameString");
                let mut buf  = [0u16; 128];
                let mut buf_size = (buf.len() * 2) as u32;
                if RegQueryValueExW(h_key, name_key.as_ptr(), null_mut(), null_mut(),
                    buf.as_mut_ptr() as *mut u8, &mut buf_size) == 0
                {
                    let len = (buf_size / 2) as usize;
                    if len > 0 {
                        let parsed  = String::from_utf16_lossy(&buf[..len - 1]);
                        let trimmed = parsed.trim().to_string();
                        if !trimmed.is_empty() { name = trimmed; }
                    }
                }
                RegCloseKey(h_key);
            }
        }

        // Fallback: query live from CallNtPowerInformation
        if mhz == 0 {
            let cores = Self::read_core_count();
            if let Some((_, max_hw)) = crate::power::PowerManager::get_live_processor_mhz(cores) {
                mhz = max_hw;
            }
        }

        // Fallback: architecture-based identity
        if name.is_empty() {
            use crate::win32::*;
            let mut sys: SYSTEM_INFO = unsafe { std::mem::zeroed() };
            unsafe { GetSystemInfo(&mut sys) };
            let arch = match sys.w_processor_architecture {
                9  => "x86_64",
                12 => "ARM64",
                0  => "x86",
                _  => "Unknown",
            };
            name = format!("{} Processor ({} Cores)", arch, sys.dw_number_of_processors);
        }

        (name, mhz)
    }

    /// Reads PC manufacturer and model from BIOS DMI tables in the Registry.
    pub fn read_system_model() -> String {
        use std::ptr::null_mut;
        use crate::win32::*;
        unsafe {
            let sub_key = to_wide("HARDWARE\\DESCRIPTION\\System\\BIOS");
            let mut h_key: usize = 0;
            if RegOpenKeyExW(0x80000002, sub_key.as_ptr(), 0, 0x20019, &mut h_key) == 0 {
                let mut mfg  = String::new();
                let mut prod = String::new();

                let mk = to_wide("SystemManufacturer");
                let mut bm = [0u16; 128];
                let mut sm = (bm.len() * 2) as u32;
                if RegQueryValueExW(h_key, mk.as_ptr(), null_mut(), null_mut(),
                    bm.as_mut_ptr() as *mut u8, &mut sm) == 0 && (sm / 2) as usize > 1
                {
                    mfg = String::from_utf16_lossy(&bm[..(sm / 2) as usize - 1]).trim().to_string();
                }

                let pk = to_wide("SystemProductName");
                let mut bp = [0u16; 128];
                let mut sp = (bp.len() * 2) as u32;
                if RegQueryValueExW(h_key, pk.as_ptr(), null_mut(), null_mut(),
                    bp.as_mut_ptr() as *mut u8, &mut sp) == 0 && (sp / 2) as usize > 1
                {
                    prod = String::from_utf16_lossy(&bp[..(sp / 2) as usize - 1]).trim().to_string();
                }

                RegCloseKey(h_key);
                return match (mfg.is_empty(), prod.is_empty()) {
                    (false, false) => format!("{} {}", mfg, prod),
                    (true,  false) => prod,
                    (false, true)  => mfg,
                    _              => "Windows PC".to_string(),
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
            if (snap as isize) == -1 || snap.is_null() { return 0; }
            let mut entry: PROCESSENTRY32W = zeroed();
            entry.dw_size = size_of::<PROCESSENTRY32W>() as u32;
            let mut count = 0u32;
            if Process32FirstW(snap, &mut entry) != 0 {
                loop {
                    count += 1;
                    if Process32NextW(snap, &mut entry) == 0 { break; }
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
                self.prev_idle   = idle.to_u64();
                self.prev_kernel = kernel.to_u64();
                self.prev_user   = user.to_u64();
            }
        }
    }

    pub fn sample_cpu_load(&mut self) -> f64 {
        use crate::win32::*;
        unsafe {
            let mut idle   = std::mem::zeroed();
            let mut kernel = std::mem::zeroed();
            let mut user   = std::mem::zeroed();
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
                let ci = idle.to_u64();
                let ck = kernel.to_u64();
                let cu = user.to_u64();

                let di = ci.saturating_sub(self.prev_idle);
                let dk = ck.saturating_sub(self.prev_kernel);
                let du = cu.saturating_sub(self.prev_user);

                self.prev_idle   = ci;
                self.prev_kernel = ck;
                self.prev_user   = cu;

                let total = dk + du;
                if total > 0 {
                    return ((total.saturating_sub(di)) as f64 / total as f64 * 100.0).clamp(0.0, 100.0);
                }
            }
        }
        0.0
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  LINUX IMPLEMENTATION
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(target_os = "linux")]
impl HardwareTelemetry {
    pub fn new() -> Self {
        let mut t = Self { prev_idle: 0, prev_kernel: 0, prev_user: 0 };
        // Prime the CPU load sampler with initial /proc/stat values
        if let Some((idle, total)) = Self::read_proc_stat() {
            t.prev_idle   = idle;
            t.prev_kernel = total; // repurpose prev_kernel as prev_total on Linux
        }
        t
    }

    /// Reads CPU model name and base frequency from /proc/cpuinfo.
    pub fn read_cpu_info() -> (String, u32) {
        let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let mut name = String::new();
        let mut mhz  = 0u32;
        for line in content.lines() {
            if line.starts_with("model name") && name.is_empty() {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    name = val.trim().to_string();
                }
            }
            if line.starts_with("cpu MHz") && mhz == 0 {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    mhz = val.trim().parse::<f64>().unwrap_or(0.0) as u32;
                }
            }
            if !name.is_empty() && mhz > 0 { break; }
        }
        if name.is_empty() { name = "Linux CPU".to_string(); }
        (name, mhz)
    }

    /// Reads system manufacturer and product name from DMI/sysfs.
    pub fn read_system_model() -> String {
        let vendor  = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor").unwrap_or_default();
        let product = std::fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default();
        let v = vendor.trim();
        let p = product.trim();
        match (v.is_empty(), p.is_empty()) {
            (false, false) => format!("{} {}", v, p),
            (true,  false) => p.to_string(),
            (false, true)  => v.to_string(),
            _              => "Linux PC".to_string(),
        }
    }

    pub fn read_core_count() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }

    /// Counts running processes by enumerating numeric entries in /proc.
    pub fn count_processes() -> u32 {
        std::fs::read_dir("/proc")
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .chars()
                            .all(|c| c.is_ascii_digit())
                    })
                    .count() as u32
            })
            .unwrap_or(0)
    }

    /// Reads (idle_jiffies, total_jiffies) from /proc/stat for the aggregate "cpu" line.
    fn read_proc_stat() -> Option<(u64, u64)> {
        let content = std::fs::read_to_string("/proc/stat").ok()?;
        let line    = content.lines().next()?; // first line = aggregate "cpu ..."
        let parts: Vec<u64> = line
            .split_whitespace()
            .skip(1) // skip "cpu" label
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        // Fields: user nice system idle iowait irq softirq steal guest guest_nice
        let idle  = parts.get(3).copied().unwrap_or(0)  // idle
                  + parts.get(4).copied().unwrap_or(0); // iowait
        let total: u64 = parts.iter().sum();
        Some((idle, total))
    }

    /// Measures CPU utilisation over the last ~1 second tick using /proc/stat deltas.
    pub fn sample_cpu_load(&mut self) -> f64 {
        if let Some((cur_idle, cur_total)) = Self::read_proc_stat() {
            let d_idle  = cur_idle.saturating_sub(self.prev_idle);
            let d_total = cur_total.saturating_sub(self.prev_kernel); // prev_kernel = prev_total

            self.prev_idle   = cur_idle;
            self.prev_kernel = cur_total;

            if d_total > 0 {
                let busy = d_total.saturating_sub(d_idle);
                return (busy as f64 / d_total as f64 * 100.0).clamp(0.0, 100.0);
            }
        }
        0.0
    }
}

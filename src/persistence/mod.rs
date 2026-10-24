use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BaselineInfo {
    pub active_scheme_guid: String,
    pub subgroup_guid: String,
    pub setting_guid: String,
    pub ac_max_percent: u32,
    pub dc_max_percent: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentStateInfo {
    pub profile: String,
    pub applied_ac_percent: u32,
    pub governor_mode: String,
}

#[derive(Serialize, Deserialize)]
pub struct RecoveryJournal {
    pub system: String,
    pub cpu: String,
    pub recorded_at: String,
    pub baseline: BaselineInfo,
    pub current_state: CurrentStateInfo,
    pub status: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_currency")]
    pub currency_symbol: String,
    #[serde(default = "default_tariff")]
    pub tariff_per_kwh: f64,
}

fn default_currency() -> String {
    "₹".to_string()
}

const fn default_tariff() -> f64 {
    7.00
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            currency_symbol: default_currency(),
            tariff_per_kwh: default_tariff(),
        }
    }
}

pub fn load_app_settings(path: &Path) -> AppSettings {
    if path.exists()
        && let Ok(s) = fs::read_to_string(path)
            && let Ok(settings) = serde_json::from_str::<AppSettings>(&s) {
                return settings;
            }
    let def = AppSettings::default();
    save_app_settings(path, &def);
    def
}

pub fn save_app_settings(path: &Path, settings: &AppSettings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

pub fn load_today_base(csv_path: &Path) -> (f64, u64, u64) {
    let (today, _, _) = get_system_date_time();
    if let Ok(mut file) = File::open(csv_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            for line in contents.lines().skip(1) {
                let cols: Vec<&str> = line.split(',').map(|s| s.trim_matches('"')).collect();
                if cols.len() >= 4 && cols[0] == today {
                    let active_mins = cols.get(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    let eco_mins = cols.get(2).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    let wh = cols.get(3).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    return (wh, (active_mins * 60.0) as u64, (eco_mins * 60.0) as u64);
                }
            }
        }
    }
    (0.0, 0, 0)
}

pub struct CsvLogStats {
    pub today_base_wh: f64,
    pub total_watt_seconds_saved: f64,
    pub tariff_per_kwh: f64,
    pub today_base_active_secs: u64,
    pub session_seconds: u64,
    pub today_base_eco_secs: u64,
    pub eco_seconds: u64,
}

pub fn flush_csv(csv_path: &Path, stats: &CsvLogStats) {
    let (today, time_str, _) = get_system_date_time();

    let total_wh = ((stats.today_base_wh + (stats.total_watt_seconds_saved / 3600.0)) * 100.0).round() / 100.0;
    let total_kwh = ((total_wh / 1000.0) * 100_000.0).round() / 100_000.0;
    let total_cost = ((total_kwh * stats.tariff_per_kwh) * 100.0).round() / 100.0;

    let total_active_mins =
        (((stats.today_base_active_secs + stats.session_seconds) as f64 / 60.0) * 10.0).round() / 10.0;
    let total_eco_mins =
        (((stats.today_base_eco_secs + stats.eco_seconds) as f64 / 60.0) * 10.0).round() / 10.0;

    let mut lines = Vec::new();
    if csv_path.exists()
        && let Ok(content) = fs::read_to_string(csv_path) {
            for line in content.lines() {
                let clean = line.trim();
                if clean.is_empty()
                    || clean.starts_with("\"Date\"")
                    || clean.starts_with("Date")
                    || clean.starts_with(&format!("\"{today}\""))
                    || clean.starts_with(&today)
                    || clean.replace(',', "").trim().is_empty()
                {
                    continue;
                }
                lines.push(clean.to_string());
            }
        }

    let new_line = format!(
        "\"{today}\",\"{total_active_mins}\",\"{total_eco_mins}\",\"{total_wh:.2}\",\"{total_kwh:.5}\",\"{total_cost:.2}\",\"{time_str}\""
    );

    let mut out = String::from(
        "\"Date\",\"ActiveMinutes\",\"EcoMinutes\",\"WattHoursSaved\",\"KwhSaved\",\"CostSaved\",\"LastUpdated\"\r\n",
    );
    for l in lines {
        out.push_str(&l);
        out.push_str("\r\n");
    }
    out.push_str(&new_line);
    out.push_str("\r\n");

    let _ = fs::write(csv_path, out);
}

#[cfg(target_os = "windows")]
pub fn get_system_date_time() -> (String, String, String) {
    use std::mem::zeroed;
    use crate::win32::*;
    unsafe {
        let mut st: SYSTEMTIME = zeroed();
        GetLocalTime(&mut st);
        let date_str = format!("{:04}-{:02}-{:02}", st.w_year, st.w_month, st.w_day);
        let time_str = format!("{:02}:{:02}:{:02}", st.w_hour, st.w_minute, st.w_second);

        let mut utc: SYSTEMTIME = zeroed();
        GetSystemTime(&mut utc);
        let iso_utc = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            utc.w_year, utc.w_month, utc.w_day, utc.w_hour, utc.w_minute, utc.w_second
        );
        (date_str, time_str, iso_utc)
    }
}

#[cfg(target_os = "linux")]
fn unix_to_ymd_hms(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let s = (secs % 60) as u32;
    let min = ((secs / 60) % 60) as u32;
    let h = ((secs / 3600) % 24) as u32;
    let mut days = (secs / 86400) as u32;

    let mut year = 1970u32;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let month_lens: [u32; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &ml in &month_lens {
        if days < ml {
            break;
        }
        days -= ml;
        month += 1;
    }
    (year, month, days + 1, h, min, s)
}

#[cfg(target_os = "linux")]
fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(target_os = "linux")]
pub fn get_system_date_time() -> (String, String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, mo, d, h, min, s) = unix_to_ymd_hms(secs);
    let date_str = format!("{y:04}-{mo:02}-{d:02}");
    let time_str = format!("{h:02}:{min:02}:{s:02}");
    let iso_utc = format!("{date_str}T{time_str}Z");
    (date_str, time_str, iso_utc)
}

#[cfg(target_os = "windows")]
pub fn update_recovery_journal(
    journal_path: &Path,
    system_name: &str,
    cpu_name: &str,
    current_limit: u32,
    profile: &str,
    status: &str,
) {
    use crate::power::PowerManager;
    use crate::win32::*;

    let scheme = PowerManager::get_active_scheme();
    let (_, _, iso_utc) = get_system_date_time();
    let journal = RecoveryJournal {
        system: system_name.to_string(),
        cpu: cpu_name.to_string(),
        recorded_at: iso_utc.clone(),
        baseline: BaselineInfo {
            active_scheme_guid: scheme.to_string_guid(),
            subgroup_guid: GUID_SUB_PROCESSOR.to_string_guid(),
            setting_guid: GUID_PROCTHROTTLEMAX.to_string_guid(),
            ac_max_percent: 100,
            dc_max_percent: 100,
        },
        current_state: CurrentStateInfo {
            profile: profile.to_string(),
            applied_ac_percent: current_limit,
            governor_mode: "Autonomous".to_string(),
        },
        status: status.to_string(),
        last_modified: iso_utc,
    };
    if let Ok(json) = serde_json::to_string_pretty(&journal) {
        let _ = fs::write(journal_path, json);
    }
}

#[cfg(target_os = "linux")]
pub fn update_recovery_journal(
    journal_path: &Path,
    system_name: &str,
    cpu_name: &str,
    current_limit: u32,
    profile: &str,
    status: &str,
) {
    let (_, _, iso_utc) = get_system_date_time();
    let journal = RecoveryJournal {
        system: system_name.to_string(),
        cpu: cpu_name.to_string(),
        recorded_at: iso_utc.clone(),
        baseline: BaselineInfo {
            active_scheme_guid: "linux-cpufreq-governor".to_string(),
            subgroup_guid: "cpu-scaling".to_string(),
            setting_guid: "scaling_max_freq".to_string(),
            ac_max_percent: 100,
            dc_max_percent: 100,
        },
        current_state: CurrentStateInfo {
            profile: profile.to_string(),
            applied_ac_percent: current_limit,
            governor_mode: "Autonomous".to_string(),
        },
        status: status.to_string(),
        last_modified: iso_utc,
    };
    if let Ok(json) = serde_json::to_string_pretty(&journal) {
        let _ = fs::write(journal_path, json);
    }
}

const APP_REG_NAME: &str = "AdaptivePowerGovernor";

#[cfg(target_os = "windows")]
pub fn is_autostart_enabled() -> bool {
    use std::ptr::null_mut;
    use crate::win32::*;
    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    unsafe {
        let sub_key = to_wide(RUN_KEY);
        let mut h: usize = 0;
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub_key.as_ptr(), 0, KEY_READ, &mut h) == 0 {
            let val = to_wide(APP_REG_NAME);
            let mut size = 0u32;
            let ok = RegQueryValueExW(h, val.as_ptr(), null_mut(), null_mut(), null_mut(), &mut size);
            RegCloseKey(h);
            ok == 0 && size > 0
        } else {
            false
        }
    }
}

#[cfg(target_os = "windows")]
pub fn set_autostart_enabled(enabled: bool, exe_path: &Path) -> bool {
    use crate::win32::*;
    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    unsafe {
        let sub_key = to_wide(RUN_KEY);
        let mut h: usize = 0;
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub_key.as_ptr(), 0, KEY_WRITE, &mut h) == 0 {
            let val_name = to_wide(APP_REG_NAME);
            let result = if enabled {
                let cmd = format!("\"{}\" --silent", exe_path.to_string_lossy());
                let wide_cmd = to_wide(&cmd);
                let byte_len = (wide_cmd.len() * 2) as u32;
                RegSetValueExW(
                    h,
                    val_name.as_ptr(),
                    0,
                    REG_SZ,
                    wide_cmd.as_ptr() as *const u8,
                    byte_len,
                ) == 0
            } else {
                let _ = RegDeleteValueW(h, val_name.as_ptr());
                true
            };
            RegCloseKey(h);
            result
        } else {
            false
        }
    }
}

#[cfg(target_os = "linux")]
pub fn is_autostart_enabled() -> bool {
    service_path().is_some_and(|p| p.exists())
}

#[cfg(target_os = "linux")]
pub fn set_autostart_enabled(enabled: bool, exe_path: &Path) -> bool {
    let Some(path) = service_path() else { return false; };
    if enabled {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let content = format!(
            "[Unit]\nDescription=Adaptive Power Governor\n\n\
             [Service]\nExecStart={exe}\nRestart=always\nRestartSec=5\n\n\
             [Install]\nWantedBy=default.target\n",
            exe = exe_path.display()
        );
        fs::write(&path, content).is_ok()
    } else {
        let _ = fs::remove_file(&path);
        true
    }
}

#[cfg(target_os = "linux")]
fn service_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        std::path::PathBuf::from(home)
            .join(".config/systemd/user/adaptive-power-governor.service"),
    )
}

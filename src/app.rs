use std::path::{Path, PathBuf};

use crate::governor::{AutonomousGovernor, PowerState};
use crate::persistence::{
    flush_csv, load_today_base, save_app_settings, update_recovery_journal, AppSettings,
    CsvLogStats,
};
use crate::power::PowerManager;
use crate::telemetry::HardwareTelemetry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsActiveField {
    Currency,
    UnitPrice,
}

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub currency: String,
    pub unit_price: String,
    pub active_field: SettingsActiveField,
    pub error_msg: Option<String>,
}

pub struct AppState {
    pub system_name: String,
    pub cpu_name: String,
    pub core_count: u32,
    pub max_mhz: u32,
    pub estimated_tdp: f64,
    pub idle_base_watts: f64,

    pub telemetry: HardwareTelemetry,
    pub governor: AutonomousGovernor,

    pub session_seconds: u64,
    pub eco_seconds: u64,
    pub total_watt_seconds_saved: f64,

    pub current_load: f64,
    pub current_mhz: u32,
    pub live_watts: f64,
    pub unthrottled_watts: f64,
    pub task_count: u32,
    pub status_headline: String,
    pub status_subtext: String,

    pub live_watts_history: Vec<f64>,
    pub unthrottled_history: Vec<f64>,

    pub today_base_wh: f64,
    pub today_base_active_secs: u64,
    pub today_base_eco_secs: u64,

    pub settings: AppSettings,
    pub settings_file_path: PathBuf,
    pub is_settings_open: bool,
    pub settings_draft: Option<SettingsDraft>,

    pub log_file_path: PathBuf,
    pub journal_file_path: PathBuf,
}

impl AppState {
    #[must_use]
    pub fn new(base_dir: &Path) -> Self {
        let system_name = HardwareTelemetry::read_system_model();
        let (cpu_name, detected_mhz) = HardwareTelemetry::read_cpu_info();
        let core_count = HardwareTelemetry::read_core_count();

        let estimated_tdp = (f64::from(core_count) * 12.0).clamp(35.0, 150.0);
        let idle_base_watts = (f64::from(core_count) * 1.5 + 4.0).clamp(8.0, 30.0);

        let max_mhz = if detected_mhz > 0 { detected_mhz } else { 3000 };

        let log_file_path = base_dir.join("daily_energy_log.csv");
        let journal_file_path = base_dir.join("recovery_journal.json");
        let settings_file_path = base_dir.join("app_settings.json");
        let settings = crate::persistence::load_app_settings(&settings_file_path);

        let (today_base_wh, today_base_active_secs, today_base_eco_secs) =
            load_today_base(&log_file_path);

        let initial_mhz = (f64::from(max_mhz) * 0.70).round() as u32;

        let app = Self {
            system_name,
            cpu_name,
            core_count,
            max_mhz,
            estimated_tdp,
            idle_base_watts,
            settings,
            settings_file_path,
            is_settings_open: false,
            settings_draft: None,
            telemetry: HardwareTelemetry::new(),
            governor: AutonomousGovernor::new(),
            session_seconds: 0,
            eco_seconds: 0,
            total_watt_seconds_saved: 0.0,
            current_load: 0.0,
            current_mhz: initial_mhz,
            live_watts: idle_base_watts,
            unthrottled_watts: idle_base_watts * 2.0,
            task_count: 0,
            status_headline: "Autonomous Mode: DEEP ECO (70% Deep Idle Cap)".to_string(),
            status_subtext: "Idle condition. Deep 70% power reduction active.".to_string(),
            live_watts_history: vec![idle_base_watts; 60],
            unthrottled_history: vec![idle_base_watts * 2.0; 60],
            today_base_wh,
            today_base_active_secs,
            today_base_eco_secs,
            log_file_path,
            journal_file_path,
        };

        PowerManager::set_limit(70);
        app.update_journal("Eco-70", "Active");
        app.flush_csv();
        app
    }

    #[must_use]
    pub const fn current_limit(&self) -> u32 {
        self.governor.current_state.limit_percent()
    }

    pub fn tick(&mut self) {
        self.session_seconds += 1;
        self.task_count = HardwareTelemetry::count_processes();
        self.current_load = self.telemetry.sample_cpu_load();

        if let Some((cur_hw_mhz, max_hw_mhz)) =
            PowerManager::get_live_processor_mhz(self.core_count)
        {
            if cur_hw_mhz > 0 {
                self.current_mhz = cur_hw_mhz;
            }
            if max_hw_mhz > self.max_mhz {
                self.max_mhz = max_hw_mhz;
            }
        } else {
            let ratio = f64::from(self.current_limit()) / 100.0;
            self.current_mhz = (f64::from(self.max_mhz) * ratio).round() as u32;
        }

        let util_fraction = (self.current_load / 100.0).clamp(0.0, 1.0);
        let live_v = if self.governor.current_state == PowerState::Boost100 {
            1.35
        } else {
            1.00
        };
        let live_freq_ratio = if self.max_mhz > 0 {
            (f64::from(self.current_mhz) / f64::from(self.max_mhz)).clamp(0.2, 1.2)
        } else {
            1.0
        };
        let dynamic_max_watts = self.estimated_tdp * 0.60;
        let live_dynamic =
            dynamic_max_watts * (live_v * live_v / (1.35 * 1.35)) * live_freq_ratio * util_fraction;

        self.live_watts =
            ((self.idle_base_watts + live_dynamic + (4.0 * live_freq_ratio)) * 10.0).round() / 10.0;
        self.unthrottled_watts =
            ((self.idle_base_watts + (dynamic_max_watts * util_fraction) + 4.0) * 10.0).round()
                / 10.0;

        let delta_watts = (self.unthrottled_watts - self.live_watts).max(0.0);
        self.total_watt_seconds_saved += delta_watts;

        if self.governor.current_state != PowerState::Boost100 {
            self.eco_seconds += 1;
        }

        let transition = self.governor.evaluate(self.current_load);
        if transition.state_changed {
            let limit = transition.new_state.limit_percent();
            PowerManager::set_limit(limit);
            self.update_journal(transition.new_state.profile_name(), "Active");
        }
        self.status_headline = transition.headline;
        self.status_subtext = transition.subtext;

        self.live_watts_history.remove(0);
        self.live_watts_history.push(self.live_watts);

        self.unthrottled_history.remove(0);
        self.unthrottled_history.push(self.unthrottled_watts);

        if self.session_seconds.is_multiple_of(60) {
            self.flush_csv();
        }
    }

    pub fn flush_csv(&self) {
        let stats = CsvLogStats {
            today_base_wh: self.today_base_wh,
            total_watt_seconds_saved: self.total_watt_seconds_saved,
            tariff_per_kwh: self.settings.tariff_per_kwh,
            today_base_active_secs: self.today_base_active_secs,
            session_seconds: self.session_seconds,
            today_base_eco_secs: self.today_base_eco_secs,
            eco_seconds: self.eco_seconds,
        };
        flush_csv(&self.log_file_path, &stats);
    }

    pub fn update_journal(&self, profile: &str, status: &str) {
        update_recovery_journal(
            &self.journal_file_path,
            &self.system_name,
            &self.cpu_name,
            self.current_limit(),
            profile,
            status,
        );
    }

    pub fn save_settings(&self) {
        save_app_settings(&self.settings_file_path, &self.settings);
    }

    pub fn open_settings(&mut self) {
        self.settings_draft = Some(SettingsDraft {
            currency: self.settings.currency_symbol.clone(),
            unit_price: format!("{:.2}", self.settings.tariff_per_kwh),
            active_field: SettingsActiveField::UnitPrice,
            error_msg: None,
        });
        self.is_settings_open = true;
    }

    pub fn close_settings(&mut self) {
        self.is_settings_open = false;
        self.settings_draft = None;
    }

    pub fn save_settings_from_draft(&mut self) -> bool {
        if let Some(draft) = &self.settings_draft {
            let cur = draft.currency.trim();
            let cur_symbol = if cur.is_empty() { "₹".to_string() } else { cur.to_string() };

            match draft.unit_price.trim().parse::<f64>() {
                Ok(val) if val > 0.0 && val <= 1000.0 => {
                    self.settings.currency_symbol = cur_symbol;
                    self.settings.tariff_per_kwh = (val * 100.0).round() / 100.0;
                    self.save_settings();
                    self.is_settings_open = false;
                    self.settings_draft = None;
                    self.flush_csv();
                    true
                }
                _ => {
                    if let Some(draft_mut) = self.settings_draft.as_mut() {
                        draft_mut.error_msg = Some("Please enter a valid price (e.g. 7.00 or 5.50)".to_string());
                    }
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.is_settings_open {
            return;
        }
        if c == '\r' {
            self.save_settings_from_draft();
            return;
        }
        if c == '\x1b' {
            self.close_settings();
            return;
        }
        if let Some(draft) = self.settings_draft.as_mut() {
            draft.error_msg = None;
            if c == '\t' {
                draft.active_field = match draft.active_field {
                    SettingsActiveField::Currency => SettingsActiveField::UnitPrice,
                    SettingsActiveField::UnitPrice => SettingsActiveField::Currency,
                };
                return;
            }
            if c == '\x08' {
                match draft.active_field {
                    SettingsActiveField::Currency => {
                        draft.currency.pop();
                    }
                    SettingsActiveField::UnitPrice => {
                        draft.unit_price.pop();
                    }
                }
                return;
            }
            if !c.is_control() {
                match draft.active_field {
                    SettingsActiveField::Currency => {
                        if draft.currency.chars().count() < 8 {
                            draft.currency.push(c);
                        }
                    }
                    SettingsActiveField::UnitPrice => {
                        if (c.is_ascii_digit() || c == '.') && draft.unit_price.chars().count() < 8
                            && !(c == '.' && draft.unit_price.contains('.')) {
                                draft.unit_price.push(c);
                            }
                    }
                }
            }
        }
    }

    pub fn handle_key_down(&mut self, key: usize) {
        if !self.is_settings_open {
            return;
        }
        match key {
            0x09 => {
                if let Some(draft) = self.settings_draft.as_mut() {
                    draft.active_field = match draft.active_field {
                        SettingsActiveField::Currency => SettingsActiveField::UnitPrice,
                        SettingsActiveField::UnitPrice => SettingsActiveField::Currency,
                    };
                }
            }
            0x0D => {
                self.save_settings_from_draft();
            }
            0x1B => {
                self.close_settings();
            }
            _ => {}
        }
    }

    pub fn handle_click(&mut self, x: i32, y: i32) {
        if !self.is_settings_open {
            if (510..=636).contains(&x) && (14..=48).contains(&y) {
                self.open_settings();
                return;
            }
            if (240..=456).contains(&x) && (180..=204).contains(&y) {
                self.open_settings();
            }
        } else {
            if !(200..=760).contains(&x) || !(70..=430).contains(&y) {
                self.close_settings();
                return;
            }
            if (700..=744).contains(&x) && (82..=118).contains(&y) {
                self.close_settings();
                return;
            }
            if (230..=730).contains(&x) && (168..=210).contains(&y) {
                if let Some(draft) = self.settings_draft.as_mut() {
                    draft.active_field = SettingsActiveField::Currency;
                    draft.error_msg = None;
                }
                return;
            }
            if (230..=730).contains(&x) && (256..=298).contains(&y) {
                if let Some(draft) = self.settings_draft.as_mut() {
                    draft.active_field = SettingsActiveField::UnitPrice;
                    draft.error_msg = None;
                }
                return;
            }
            if (350..=470).contains(&x) && (356..=398).contains(&y) {
                self.close_settings();
                return;
            }
            if (490..=730).contains(&x) && (356..=398).contains(&y) {
                self.save_settings_from_draft();
            }
        }
    }
}

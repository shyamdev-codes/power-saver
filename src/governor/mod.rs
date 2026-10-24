#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    DeepEco70,
    Balanced80,
    Boost100,
}

impl PowerState {
    pub fn limit_percent(&self) -> u32 {
        match self {
            PowerState::DeepEco70 => 70,
            PowerState::Balanced80 => 80,
            PowerState::Boost100 => 100,
        }
    }

    pub fn profile_name(&self) -> &'static str {
        match self {
            PowerState::DeepEco70 => "Eco-70",
            PowerState::Balanced80 => "Balanced-80",
            PowerState::Boost100 => "Boost-100",
        }
    }
}

pub struct AutonomousGovernor {
    pub current_state: PowerState,
    pub idle_seconds_count: u32,
}

pub struct GovernorTransition {
    pub new_state: PowerState,
    pub state_changed: bool,
    pub headline: String,
    pub subtext: String,
}

impl AutonomousGovernor {
    pub fn new() -> Self {
        Self {
            current_state: PowerState::DeepEco70,
            idle_seconds_count: 0,
        }
    }

    pub fn evaluate(&mut self, current_load: f64) -> GovernorTransition {
        if current_load >= 70.0 {
            self.idle_seconds_count = 0;
            let changed = self.current_state != PowerState::Boost100;
            self.current_state = PowerState::Boost100;
            GovernorTransition {
                new_state: PowerState::Boost100,
                state_changed: changed,
                headline: "Autonomous Mode: BOOST (100% Uncapped Limit)".to_string(),
                subtext: format!(
                    "High workload demand ({:.0}%). Escalated to 100% full throughput.",
                    current_load
                ),
            }
        } else if current_load >= 35.0 {
            self.idle_seconds_count = 0;
            let changed = self.current_state != PowerState::Balanced80;
            let sub = if self.current_state == PowerState::DeepEco70 {
                format!(
                    "Active workload ({:.0}%). Stepped to 80% for smooth interactive response.",
                    current_load
                )
            } else {
                format!(
                    "Work settling ({:.0}%). Operating at 80% balanced capacity.",
                    current_load
                )
            };
            self.current_state = PowerState::Balanced80;
            GovernorTransition {
                new_state: PowerState::Balanced80,
                state_changed: changed,
                headline: "Autonomous Mode: BALANCED (80% Power Limit)".to_string(),
                subtext: sub,
            }
        } else {
            // Calm / Idle (< 35% load)
            self.idle_seconds_count += 1;
            if self.current_state == PowerState::Boost100 {
                self.current_state = PowerState::Balanced80;
                GovernorTransition {
                    new_state: PowerState::Balanced80,
                    state_changed: true,
                    headline: "Autonomous Mode: BALANCED (80% Power Limit)".to_string(),
                    subtext: format!(
                        "Burst completed ({:.0}%). Settling back to 80% balanced.",
                        current_load
                    ),
                }
            } else if self.current_state == PowerState::Balanced80 && self.idle_seconds_count >= 3 {
                self.current_state = PowerState::DeepEco70;
                GovernorTransition {
                    new_state: PowerState::DeepEco70,
                    state_changed: true,
                    headline: "Autonomous Mode: DEEP ECO (70% Deep Idle Cap)".to_string(),
                    subtext: format!(
                        "Idle condition sustained ({:.0}% load). Deep 70% power reduction active.",
                        current_load
                    ),
                }
            } else {
                // Keeping current state (no change)
                let (h, s) = match self.current_state {
                    PowerState::DeepEco70 => (
                        "Autonomous Mode: DEEP ECO (70% Deep Idle Cap)".to_string(),
                        format!(
                            "Idle condition sustained ({:.0}% load). Deep 70% power reduction active.",
                            current_load
                        ),
                    ),
                    PowerState::Balanced80 => (
                        "Autonomous Mode: BALANCED (80% Power Limit)".to_string(),
                        format!(
                            "Calm workload ({:.0}%). Maintaining 80% (idle stabilization: {}/3s).",
                            current_load, self.idle_seconds_count
                        ),
                    ),
                    PowerState::Boost100 => (
                        "Autonomous Mode: BOOST (100% Uncapped Limit)".to_string(),
                        format!("High demand ({:.0}%). Uncapped full performance.", current_load),
                    ),
                };
                GovernorTransition {
                    new_state: self.current_state,
                    state_changed: false,
                    headline: h,
                    subtext: s,
                }
            }
        }
    }
}

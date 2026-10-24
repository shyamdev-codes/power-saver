#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PowerState {
    #[default]
    DeepEco70,
    Balanced80,
    Boost100,
}

impl PowerState {
    #[must_use]
    pub const fn limit_percent(&self) -> u32 {
        match self {
            Self::DeepEco70 => 70,
            Self::Balanced80 => 80,
            Self::Boost100 => 100,
        }
    }

    #[must_use]
    pub const fn profile_name(&self) -> &'static str {
        match self {
            Self::DeepEco70 => "Eco-70",
            Self::Balanced80 => "Balanced-80",
            Self::Boost100 => "Boost-100",
        }
    }
}

#[derive(Debug, Default)]
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&mut self, current_load: f64) -> GovernorTransition {
        if current_load >= 70.0 {
            self.idle_seconds_count = 0;
            let changed = self.current_state != PowerState::Boost100;
            self.current_state = PowerState::Boost100;

            GovernorTransition {
                new_state: PowerState::Boost100,
                state_changed: changed,
                headline: "Autonomous Mode: BOOST (100% Limit)".to_string(),
                subtext: format!("High load demand ({current_load:.0}%). Scaled to 100% capacity."),
            }
        } else if current_load >= 35.0 {
            self.idle_seconds_count = 0;
            let changed = self.current_state != PowerState::Balanced80;
            let subtext = if self.current_state == PowerState::DeepEco70 {
                format!("Active workload ({current_load:.0}%). Raised to 80% responsive capacity.")
            } else {
                format!("Work settling ({current_load:.0}%). Maintaining 80% balanced capacity.")
            };
            self.current_state = PowerState::Balanced80;

            GovernorTransition {
                new_state: PowerState::Balanced80,
                state_changed: changed,
                headline: "Autonomous Mode: BALANCED (80% Limit)".to_string(),
                subtext,
            }
        } else {
            self.idle_seconds_count += 1;

            if self.current_state == PowerState::Boost100 {
                self.current_state = PowerState::Balanced80;
                GovernorTransition {
                    new_state: PowerState::Balanced80,
                    state_changed: true,
                    headline: "Autonomous Mode: BALANCED (80% Limit)".to_string(),
                    subtext: format!("Burst ended ({current_load:.0}%). Settling back to 80% balanced."),
                }
            } else if self.current_state == PowerState::Balanced80 && self.idle_seconds_count >= 3 {
                self.current_state = PowerState::DeepEco70;
                GovernorTransition {
                    new_state: PowerState::DeepEco70,
                    state_changed: true,
                    headline: "Autonomous Mode: DEEP ECO (70% Limit)".to_string(),
                    subtext: format!("Idle sustained ({current_load:.0}%). Deep 70% energy cap active."),
                }
            } else {
                let (headline, subtext) = match self.current_state {
                    PowerState::DeepEco70 => (
                        "Autonomous Mode: DEEP ECO (70% Limit)".to_string(),
                        format!("Idle sustained ({current_load:.0}%). Deep 70% energy cap active."),
                    ),
                    PowerState::Balanced80 => (
                        "Autonomous Mode: BALANCED (80% Limit)".to_string(),
                        format!("Calm load ({current_load:.0}%). At 80% (idle delay: {}/3s).", self.idle_seconds_count),
                    ),
                    PowerState::Boost100 => (
                        "Autonomous Mode: BOOST (100% Limit)".to_string(),
                        format!("Peak demand ({current_load:.0}%). Uncapped performance."),
                    ),
                };

                GovernorTransition {
                    new_state: self.current_state,
                    state_changed: false,
                    headline,
                    subtext,
                }
            }
        }
    }
}

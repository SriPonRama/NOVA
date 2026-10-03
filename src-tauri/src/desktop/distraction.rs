use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistractionState {
    Idle,
    FocusActive,
    RelevantActivity,
    PotentialDistraction,
    DistractionConfirmed,
    InterventionShown,
    Cooldown,
}

pub struct DistractionEngine {
    pub current_state: DistractionState,
    pub potential_distraction_started_at: Option<i64>,
    pub cooldown_started_at: Option<i64>,
}

impl DistractionEngine {
    pub fn new() -> Self {
        Self {
            current_state: DistractionState::Idle,
            potential_distraction_started_at: None,
            cooldown_started_at: None,
        }
    }

    pub fn reset(&mut self) {
        self.current_state = DistractionState::Idle;
        self.potential_distraction_started_at = None;
        self.cooldown_started_at = None;
    }

    pub fn update(
        &mut self,
        is_focus_active: bool,
        is_assessment_mode: bool,
        is_desktop_awareness_enabled: bool,
        current_category: &super::activity::ActivityCategory,
        current_time: i64,
        grace_period_seconds: u32,
        cooldown_minutes: u32,
    ) -> DistractionState {
        if is_assessment_mode || !is_desktop_awareness_enabled || !is_focus_active {
            self.reset();
            return self.current_state.clone();
        }

        // We are in a focus session, so if we were idle, move to FocusActive
        if self.current_state == DistractionState::Idle {
            self.current_state = DistractionState::FocusActive;
        }

        // Check if cooldown expired
        if self.current_state == DistractionState::Cooldown {
            if let Some(started_at) = self.cooldown_started_at {
                if current_time >= started_at + (cooldown_minutes as i64 * 60) {
                    self.current_state = DistractionState::FocusActive;
                    self.cooldown_started_at = None;
                } else {
                    // Still in cooldown, don't change state based on category
                    return self.current_state.clone();
                }
            } else {
                self.current_state = DistractionState::FocusActive;
            }
        }

        // If intervention is shown, we wait for user response, don't change state
        if self.current_state == DistractionState::InterventionShown {
            return self.current_state.clone();
        }

        match current_category {
            super::activity::ActivityCategory::Distracting => {
                if self.current_state == DistractionState::FocusActive || self.current_state == DistractionState::RelevantActivity {
                    self.current_state = DistractionState::PotentialDistraction;
                    self.potential_distraction_started_at = Some(current_time);
                } else if self.current_state == DistractionState::PotentialDistraction {
                    if let Some(started_at) = self.potential_distraction_started_at {
                        if current_time >= started_at + grace_period_seconds as i64 {
                            self.current_state = DistractionState::DistractionConfirmed;
                        }
                    }
                }
            }
            super::activity::ActivityCategory::Study | super::activity::ActivityCategory::Productive | super::activity::ActivityCategory::Neutral => {
                self.current_state = DistractionState::RelevantActivity;
                self.potential_distraction_started_at = None;
            }
            super::activity::ActivityCategory::Unknown => {
                // Do not change state immediately to avoid false positives, but clear potential distraction.
                if self.current_state == DistractionState::PotentialDistraction {
                    self.current_state = DistractionState::FocusActive;
                    self.potential_distraction_started_at = None;
                }
            }
        }

        self.current_state.clone()
    }

    pub fn set_intervention_shown(&mut self) {
        if self.current_state == DistractionState::DistractionConfirmed {
            self.current_state = DistractionState::InterventionShown;
        }
    }

    pub fn return_to_focus(&mut self) {
        self.current_state = DistractionState::FocusActive;
        self.potential_distraction_started_at = None;
    }

    pub fn keep_working_here(&mut self, current_time: i64) {
        self.current_state = DistractionState::Cooldown;
        self.cooldown_started_at = Some(current_time);
        self.potential_distraction_started_at = None;
    }
}

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use super::service::DrowsinessSignal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DrowsinessState {
    Awake,
    PossiblyDrowsy,
    Drowsy,
    Intervention,
    Cooldown,
}

pub struct DrowsinessStateMachine {
    pub state: DrowsinessState,
    
    // Configurable thresholds
    pub ear_threshold: f64,
    pub confirmation_duration_secs: u64,
    pub recovery_duration_secs: u64,
    pub cooldown_duration_secs: u64,
    
    // Internal trackers
    low_ear_start_time: Option<u64>,
    recovery_start_time: Option<u64>,
    cooldown_end_time: Option<u64>,
}

impl Default for DrowsinessStateMachine {
    fn default() -> Self {
        Self {
            state: DrowsinessState::Awake,
            ear_threshold: 0.22, // Sensible EAR threshold
            confirmation_duration_secs: 5,
            recovery_duration_secs: 3,
            cooldown_duration_secs: 300, // 5 minutes cooldown
            
            low_ear_start_time: None,
            recovery_start_time: None,
            cooldown_end_time: None,
        }
    }
}

impl DrowsinessStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    fn now_secs() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    #[cfg(test)]
    fn now_secs_override(&self, time: u64) -> u64 {
        time
    }

    pub fn reset(&mut self) {
        self.state = DrowsinessState::Awake;
        self.low_ear_start_time = None;
        self.recovery_start_time = None;
        self.cooldown_end_time = None;
    }

    pub fn set_cooldown(&mut self) {
        self.state = DrowsinessState::Cooldown;
        self.cooldown_end_time = Some(Self::now_secs() + self.cooldown_duration_secs);
        self.low_ear_start_time = None;
        self.recovery_start_time = None;
    }

    pub fn process_signal(&mut self, signal: &DrowsinessSignal) -> DrowsinessState {
        self.process_signal_with_time(signal, Self::now_secs())
    }

    fn process_signal_with_time(&mut self, signal: &DrowsinessSignal, now: u64) -> DrowsinessState {
        // 1. Check Cooldown
        if let DrowsinessState::Cooldown = self.state {
            if let Some(end_time) = self.cooldown_end_time {
                if now >= end_time {
                    self.reset();
                } else {
                    return self.state.clone();
                }
            }
        }

        // 2. Validate signal
        if signal.confidence < 0.5 || !signal.face_detected || signal.eye_measurement.is_nan() || signal.eye_measurement < 0.0 || signal.eye_measurement > 2.0 {
            // Signal is invalid or face lost. Don't immediately penalize or reward.
            // Reset recovery since we don't know they are awake.
            // Keep low_ear_start_time if we were already evaluating so a brief blink or lost face doesn't restart it completely.
            self.recovery_start_time = None;
            return self.state.clone();
        }

        // 3. Temporal evaluation
        if signal.eye_measurement <= self.ear_threshold {
            // Drowsiness evidence
            self.recovery_start_time = None;

            if self.low_ear_start_time.is_none() {
                self.low_ear_start_time = Some(now);
            }

            if let Some(start) = self.low_ear_start_time {
                let elapsed = now.saturating_sub(start);
                
                if elapsed >= self.confirmation_duration_secs {
                    if self.state != DrowsinessState::Intervention {
                        self.state = DrowsinessState::Drowsy;
                    }
                } else if elapsed >= self.confirmation_duration_secs / 2 {
                    if self.state == DrowsinessState::Awake {
                        self.state = DrowsinessState::PossiblyDrowsy;
                    }
                }
            }
        } else {
            // Awake evidence
            self.low_ear_start_time = None; // Cancel drowsiness evaluation

            if self.state != DrowsinessState::Awake {
                if self.recovery_start_time.is_none() {
                    self.recovery_start_time = Some(now);
                }

                if let Some(start) = self.recovery_start_time {
                    // For intervention, require double recovery time naturally without dismissing
                    let req_duration = if self.state == DrowsinessState::Intervention {
                        self.recovery_duration_secs * 2
                    } else {
                        self.recovery_duration_secs
                    };

                    if now.saturating_sub(start) >= req_duration {
                        self.reset();
                    }
                }
            }
        }

        // Transition from Drowsy to Intervention immediately
        if self.state == DrowsinessState::Drowsy {
            self.state = DrowsinessState::Intervention;
        }

        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_signal(ear: f64) -> DrowsinessSignal {
        DrowsinessSignal {
            face_detected: true,
            eye_measurement: ear,
            confidence: 0.9,
            timestamp: 0,
            status: "OK".to_string(),
        }
    }

    #[test]
    fn test_awake_remains_awake() {
        let mut sm = DrowsinessStateMachine::new();
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.3), 10), DrowsinessState::Awake);
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.3), 20), DrowsinessState::Awake);
    }

    #[test]
    fn test_one_low_ear_does_not_trigger_drowsiness() {
        let mut sm = DrowsinessStateMachine::new();
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 10), DrowsinessState::Awake);
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.3), 11), DrowsinessState::Awake);
    }

    #[test]
    fn test_sustained_low_ear_triggers_possibly_drowsy_and_intervention() {
        let mut sm = DrowsinessStateMachine::new();
        sm.confirmation_duration_secs = 4;
        
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 10), DrowsinessState::Awake); // start
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 12), DrowsinessState::PossiblyDrowsy); // 2 secs elapsed
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 14), DrowsinessState::Intervention); // 4 secs elapsed (Drowsy -> Intervention)
    }

    #[test]
    fn test_recovery_returns_to_awake() {
        let mut sm = DrowsinessStateMachine::new();
        sm.confirmation_duration_secs = 4;
        sm.recovery_duration_secs = 3;

        sm.process_signal_with_time(&dummy_signal(0.1), 10);
        sm.process_signal_with_time(&dummy_signal(0.1), 14);
        assert_eq!(sm.state, DrowsinessState::Intervention);

        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.3), 15), DrowsinessState::Intervention); // start recovery
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.3), 20), DrowsinessState::Awake); // 5 secs recovered (> 3 * 2)
    }

    #[test]
    fn test_brief_face_disappearance_no_trigger() {
        let mut sm = DrowsinessStateMachine::new();
        sm.process_signal_with_time(&dummy_signal(0.1), 10);
        
        let mut invalid_sig = dummy_signal(0.1);
        invalid_sig.face_detected = false;
        assert_eq!(sm.process_signal_with_time(&invalid_sig, 15), DrowsinessState::Awake);
    }

    #[test]
    fn test_invalid_ear_rejected() {
        let mut sm = DrowsinessStateMachine::new();
        sm.process_signal_with_time(&dummy_signal(0.1), 10);
        
        let invalid_sig = dummy_signal(-1.0);
        assert_eq!(sm.process_signal_with_time(&invalid_sig, 20), DrowsinessState::Awake);
    }

    #[test]
    fn test_cooldown_suppresses() {
        let mut sm = DrowsinessStateMachine::new();
        sm.confirmation_duration_secs = 4;
        sm.cooldown_duration_secs = 60;
        sm.process_signal_with_time(&dummy_signal(0.1), 10);
        sm.process_signal_with_time(&dummy_signal(0.1), 14);
        assert_eq!(sm.state, DrowsinessState::Intervention);
        
        sm.set_cooldown(); // simulates User clicking "I'm fine" (time inside will use real time sadly due to implementation, we can mock it by forcing state)
        
        // Manual override for testing
        sm.state = DrowsinessState::Cooldown;
        sm.cooldown_end_time = Some(100);
        
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 50), DrowsinessState::Cooldown);
        assert_eq!(sm.process_signal_with_time(&dummy_signal(0.1), 101), DrowsinessState::Awake); // Cooldown expired, evaluates again
    }
}

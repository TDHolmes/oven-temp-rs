//! State-machine for when to display the temperature and when to conserve power.

/// Threshold at which we start displaying the temperature
const TEMP_ON_THRESHOLD: f32 = 100.;
/// Threshold at which we turn the display back off as the oven cools off
const TEMP_OFF_THRESHOLD: f32 = 300.;
/// Some hysteresis to avoid thrash
const TEMP_HYSTERESIS: f32 = 10.;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum OvenTempState {
    /// The oven is determined to be off and not running
    Off,
    /// The oven is heating up and we should start displaying the temperature
    HeatingUp,
    /// The oven is sufficiently hot enough to start potentially detecting a cool-down
    AtTemp,
    /// The oven is cooling off and we should stop displaying the temp
    CoolingDown,
}

/// Structure to keep track of our oven temp state
pub struct OvenTemp {
    /// The current state of our oven
    pub state: OvenTempState,
}

impl OvenTemp {
    /// Creates a new `OvenTemp` object
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: OvenTempState::AtTemp,
        }
    }

    /// Checks for state transitions given the new temperature information
    ///
    /// # Arguments
    /// * `temp`: The new temperature reading
    ///
    /// # Returns
    /// the new oven temp state, if a transition occurred
    pub fn check_transition(&mut self, temp: f32) -> Option<OvenTempState> {
        // Potentially move to a new state
        let new_state_opt = match self.state {
            OvenTempState::Off => {
                if temp >= TEMP_ON_THRESHOLD + TEMP_HYSTERESIS {
                    Some(OvenTempState::HeatingUp)
                } else {
                    None
                }
            }
            OvenTempState::HeatingUp => {
                if temp >= TEMP_OFF_THRESHOLD + TEMP_HYSTERESIS {
                    Some(OvenTempState::AtTemp)
                } else if temp < TEMP_ON_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempState::Off)
                } else {
                    None
                }
            }
            OvenTempState::AtTemp => {
                if temp <= TEMP_OFF_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempState::CoolingDown)
                } else {
                    None
                }
            }
            OvenTempState::CoolingDown => {
                if temp <= TEMP_ON_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempState::Off)
                } else if temp >= TEMP_OFF_THRESHOLD + TEMP_HYSTERESIS {
                    Some(OvenTempState::AtTemp)
                } else {
                    None
                }
            }
        };

        if let Some(new_state) = new_state_opt {
            self.state = new_state;
        }

        new_state_opt
    }
}

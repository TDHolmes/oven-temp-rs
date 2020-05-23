/// Threshold at which we start displaying the temperature
const TEMP_ON_THRESHOLD: f32 = 100.;
/// Threshold at which we turn the display back off as the oven cools off
const TEMP_OFF_THRESHOLD: f32 = 200.;
/// Some hysteresis to avoid thrash
const TEMP_HYSTERESIS: f32 = 10.;

#[derive(Copy, Clone)]
pub enum OvenTempState {
    Off,
    HeatingUp,
    AtTemp,
    CoolingDown,
}

pub struct OvenTemp {
    pub state: OvenTempState,
    temp_previous: f32,
}

impl OvenTemp {
    pub fn new() -> OvenTemp {
        OvenTemp {
            state: OvenTempState::AtTemp,
            temp_previous: 0.,
        }
    }

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
                } else if temp >= TEMP_ON_THRESHOLD - TEMP_HYSTERESIS {
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
                } else {
                    None
                }
            }
        };

        if let Some(new_state) = new_state_opt {
            self.state = new_state;
        }

        self.temp_previous = temp;
        new_state_opt
    }
}

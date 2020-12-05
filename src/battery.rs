//! Estimates battery percentage based on voltage. Got the numbers from https://blog.ampow.com/lipo-voltage-chart/

/// Table that maps battery voltage to percentage. Each voltage step is 5% of battery capacity (95% - 0%)
const BATTERY_VOLTAGE: [f32; 20] = [
    4.15,
    4.11,
    4.08,
    4.02,
    3.98,
    3.95,
    3.91,
    3.87,
    3.85,
    3.84,
    3.82,
    3.80,
    3.79,
    3.77,
    3.75,
    3.73,
    3.71,
    3.70,  // Don't discharge below this amount for good battery health
    3.60,
    3.20,
];

/// Converts the given battery voltage to an estimated battery percentage
pub fn voltage_to_percentage(voltage: f32) -> usize {
    let mut percentage: usize = 100;
    for lut_voltage in &BATTERY_VOLTAGE {
        if voltage >= *lut_voltage {
            break;
        }
        percentage -= 5;
    }
    percentage
}

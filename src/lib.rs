use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[cfg(all(feature = "panic-hook", target_arch = "wasm32"))]
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(any(not(feature = "panic-hook"), not(target_arch = "wasm32")))]
fn init_panic_hook() {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FluxgateReading {
    pub timestamp_ms: f64,
    pub field_strength: f64,
    pub temperature_c: f64,
}

impl FluxgateReading {
    pub fn calibrated(&self, calibration: &FluxgateCalibration) -> FluxgateReading {
        let mut reading = self.clone();
        reading.field_strength = (reading.field_strength + calibration.offset) * calibration.scale;
        reading
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FluxgateCalibration {
    pub offset: f64,
    pub scale: f64,
}

#[derive(Debug, Error)]
enum FluxgateError {
    #[error("no readings available")]
    NoReadings,
    #[error("failed to deserialize: {0}")]
    Deserialize(String),
    #[error("failed to serialize: {0}")]
    Serialize(String),
}

impl FluxgateError {
    fn into_js(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

#[wasm_bindgen]
pub struct FluxgateSensor {
    readings: Vec<FluxgateReading>,
}

#[wasm_bindgen]
impl FluxgateSensor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FluxgateSensor {
        init_panic_hook();

        FluxgateSensor {
            readings: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn push_reading(&mut self, reading: JsValue) -> Result<(), JsValue> {
        let reading: FluxgateReading = reading
            .into_serde()
            .map_err(|err| FluxgateError::Deserialize(err.to_string()).into_js())?;
        self.readings.push(reading);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn readings(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.readings)
            .map_err(|err| FluxgateError::Serialize(err.to_string()).into_js())
    }

    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.readings.len()
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.readings.clear();
    }

    #[wasm_bindgen]
    pub fn average_field(&self) -> Result<f64, JsValue> {
        if self.readings.is_empty() {
            return Err(FluxgateError::NoReadings.into_js());
        }

        let total: f64 = self
            .readings
            .iter()
            .map(|reading| reading.field_strength)
            .sum();
        Ok(total / self.readings.len() as f64)
    }

    #[wasm_bindgen]
    pub fn latest(&self) -> Result<JsValue, JsValue> {
        let reading = self
            .readings
            .last()
            .ok_or_else(|| FluxgateError::NoReadings.into_js())?;
        JsValue::from_serde(reading)
            .map_err(|err| FluxgateError::Serialize(err.to_string()).into_js())
    }
}

#[wasm_bindgen]
pub fn apply_calibration(reading: JsValue, calibration: JsValue) -> Result<JsValue, JsValue> {
    let mut reading: FluxgateReading = reading
        .into_serde()
        .map_err(|err| FluxgateError::Deserialize(err.to_string()).into_js())?;
    let calibration: FluxgateCalibration = calibration
        .into_serde()
        .map_err(|err| FluxgateError::Deserialize(err.to_string()).into_js())?;

    reading.field_strength = (reading.field_strength + calibration.offset) * calibration.scale;

    JsValue::from_serde(&reading)
        .map_err(|err| FluxgateError::Serialize(err.to_string()).into_js())
}

pub fn calibrate_readings(
    readings: &[FluxgateReading],
    calibration: &FluxgateCalibration,
) -> Vec<FluxgateReading> {
    readings
        .iter()
        .map(|reading| reading.calibrated(calibration))
        .collect()
}

#[wasm_bindgen]
pub fn calibrate_series(readings: JsValue, calibration: JsValue) -> Result<JsValue, JsValue> {
    let readings: Vec<FluxgateReading> = readings
        .into_serde()
        .map_err(|err| FluxgateError::Deserialize(err.to_string()).into_js())?;
    let calibration: FluxgateCalibration = calibration
        .into_serde()
        .map_err(|err| FluxgateError::Deserialize(err.to_string()).into_js())?;

    let calibrated = calibrate_readings(&readings, &calibration);

    JsValue::from_serde(&calibrated)
        .map_err(|err| FluxgateError::Serialize(err.to_string()).into_js())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_reading(field_strength: f64) -> FluxgateReading {
        FluxgateReading {
            timestamp_ms: 1_700_000_000_000.0,
            field_strength,
            temperature_c: 21.5,
        }
    }

    #[test]
    fn calibration_is_applied() {
        let reading = sample_reading(30.0);
        let calibration = FluxgateCalibration {
            offset: -2.0,
            scale: 1.1,
        };

        let calibrated = reading.calibrated(&calibration);
        assert!((calibrated.field_strength - 30.8).abs() < f64::EPSILON);
    }

    #[test]
    fn sensor_tracks_average() {
        let mut sensor = FluxgateSensor::new();

        for strength in [25.0, 30.0, 35.0] {
            let value = JsValue::from_serde(&sample_reading(strength)).unwrap();
            sensor.push_reading(value).unwrap();
        }

        let average = sensor.average_field().unwrap();
        assert!((average - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn js_roundtrip() {
        let reading = sample_reading(25.0);
        let calibration = FluxgateCalibration {
            offset: 1.0,
            scale: 0.5,
        };

        let reading_js = JsValue::from_serde(&reading).unwrap();
        let calibration_js = JsValue::from_serde(&calibration).unwrap();
        let calibrated = apply_calibration(reading_js, calibration_js).unwrap();
        let calibrated: FluxgateReading = calibrated.into_serde().unwrap();

        assert!((calibrated.field_strength - 13.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calibrate_series_handles_vectors() {
        let readings = vec![sample_reading(10.0), sample_reading(20.0)];
        let calibration = FluxgateCalibration {
            offset: 0.0,
            scale: 2.0,
        };

        let result = calibrate_readings(&readings, &calibration);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].field_strength, 20.0);
        assert_eq!(result[1].field_strength, 40.0);

        let readings_js = JsValue::from_serde(&readings).unwrap();
        let calibration_js = JsValue::from_serde(&calibration).unwrap();
        let result_js = calibrate_series(readings_js, calibration_js).unwrap();
        let result_vec: Vec<FluxgateReading> = result_js.into_serde().unwrap();
        assert_eq!(result_vec, result);
    }

    #[test]
    fn average_requires_readings() {
        let sensor = FluxgateSensor::new();
        let err = sensor.average_field().unwrap_err();
        assert!(err.as_string().unwrap().contains("no readings"));
    }

    #[test]
    fn serialization_errors_are_forwarded() {
        let mut sensor = FluxgateSensor::new();
        let reading = json!({ "timestamp_ms": 1, "field_strength": "bad", "temperature_c": 0 });
        let err = sensor
            .push_reading(JsValue::from_serde(&reading).unwrap())
            .unwrap_err();
        assert!(err
            .as_string()
            .unwrap()
            .contains("failed to deserialize"));
    }
}

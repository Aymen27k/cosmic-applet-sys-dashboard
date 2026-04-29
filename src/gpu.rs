use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

pub fn get_gpu_temp() -> String {
    // 1. Initialize the Nvidia Management Library
    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(_) => return "Nvidia Driver Not Found".to_string(),
    };

    // 2. Get the first GPU (index 0)
    match nvml.device_by_index(0) {
        Ok(device) => {
            // 3. Ask the sensor for the temperature
            match device.temperature(TemperatureSensor::Gpu) {
                Ok(temp) => format!("{:.1}°C", temp),
                Err(_) => "Sensor Error".to_string(),
            }
        }
        Err(_) => "No GPU Found".to_string(),
    }
}
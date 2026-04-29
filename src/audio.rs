use std::fs;

pub fn get_audio_info() -> String {
    let path = "/proc/asound/Creative/pcm0p/sub0/hw_params";
    
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.trim() == "closed" {
                return "Inactive".to_string();
            }

            // Look for the rate line
            content.lines()
                .find(|line| line.starts_with("rate:"))
                .and_then(|line| {
                    // Split "rate: 44100 (44100/1)"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.get(1).and_then(|raw| raw.parse::<f32>().ok())
                })
                .map(|num| format!("{:.1} kHz", num / 1000.0))
                .unwrap_or_else(|| "Unknown".to_string())
        }
        Err(_) => "Not Found".to_string(),
    }
}
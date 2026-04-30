use sysinfo::{System, Networks, Components};
use crate::gpu::get_gpu_temp;
use crate::audio::get_audio_info;

pub struct SystemState {
    pub cpu_load: i32,
    pub cpu_temp: f32,
    pub gpu_temp: String,
    pub ram_usage: (u64, u64),
    pub audio_rate: String,
    pub net_down: u64,
    pub net_up: u64,
}

pub fn collect_system_data(sys: &mut System, components: &mut Components, last_rx: &mut u64, last_tx: &mut u64) -> SystemState {
    sys.refresh_all();
    components.refresh(false);
    
    // 1. Network Logic: More concise lookup
    let networks = Networks::new_with_refreshed_list();
    let (download_speed, upload_speed) = networks.get("enp3s0")
        .map(|data| {
            let current_rx = data.total_received();
            let current_tx = data.total_transmitted();
            
            let rx_diff = (current_rx.saturating_sub(*last_rx)) / 1024;
            let tx_diff = (current_tx.saturating_sub(*last_tx)) / 1024;
            
            *last_rx = current_rx;
            *last_tx = current_tx;
            
            (rx_diff, tx_diff)
        })
        .unwrap_or((0, 0));

    // 2. CPU Temp: Using .find() instead of a manual loop
    let cpu_t = components.iter()
        .find(|c| c.label() == "coretemp Package id 0")
        .and_then(|c| c.temperature())
        .unwrap_or(0.0);

    SystemState {
        cpu_load: sys.global_cpu_usage() as i32,
        cpu_temp: cpu_t,
        gpu_temp: get_gpu_temp(),
        ram_usage: (
            sys.used_memory() / 1024 / 1024 / 1024, 
            (sys.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0).round() as u64
        ),
        audio_rate: get_audio_info(),
        net_down: download_speed,
        net_up: upload_speed,
    }
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            cpu_load: 0,
            cpu_temp: 0.0,
            gpu_temp: String::from("0°C"),
            ram_usage: (0, 0),
            audio_rate: String::from("N/A"),
            net_down: 0,
            net_up: 0,
        }
    }

    pub fn format_net_down(&self) -> String {
        Self::format_speed(self.net_down)
    }

    pub fn format_net_up(&self) -> String {
        Self::format_speed(self.net_up)
    }

    fn format_speed(kb_per_sec: u64) -> String {
        if kb_per_sec >= 1024 {
            format!("{:.1} MB/s", kb_per_sec as f32 / 1024.0)
        } else {
            format!("{} KB/s", kb_per_sec)
        }
    }
}
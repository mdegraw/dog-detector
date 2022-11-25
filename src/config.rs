use serde::Deserialize;

fn mqtt_host_default() -> String {
    "localhost".to_string()
}

fn mqtt_port_default() -> u16 {
    1883
}

fn detector_threshold_default() -> f32 {
    0.7
}

fn stream_duration_default() -> u64 {
    30
}

fn pause_duration_default() -> u64 {
    90
}

fn camera_index_default() -> usize {
    0
}

fn camera_fps_default() -> u32 {
    30
}

fn oled_threshold_default() -> u8 {
    30
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "mqtt_host_default")]
    pub mqtt_host: String,
    #[serde(default = "mqtt_port_default")]
    pub mqtt_port: u16,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    #[serde(default = "detector_threshold_default")]
    pub detector_threshold: f32,
    #[serde(default = "stream_duration_default")]
    pub stream_duration: u64,
    #[serde(default = "pause_duration_default")]
    pub pause_duration: u64,
    #[serde(default = "camera_index_default")]
    pub camera_index: usize,
    #[serde(default = "camera_fps_default")]
    pub camera_fps: u32,
    #[serde(default = "oled_threshold_default")]
    pub oled_threshold: u8,
    /// must be absolute path to file
    pub tensorflow_model_file: String,
}

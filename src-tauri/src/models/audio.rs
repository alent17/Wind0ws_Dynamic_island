use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemAudioState {
    pub volume_percent: u8,
    pub muted: bool,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppAudioSession {
    pub process_id: u32,
    pub device_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub volume_percent: u8,
    pub muted: bool,
    pub system_sounds: bool,
}

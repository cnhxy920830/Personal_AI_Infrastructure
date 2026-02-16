use crate::{AppState, Settings};
use std::fs;
use std::path::PathBuf;
use tauri::State;

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
}

fn get_settings_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let path = get_settings_path();
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;

    Ok(())
}

pub fn load_settings_from_disk() -> Result<Settings, String> {
    let path = get_settings_path();
    
    if !path.exists() {
        return Ok(Settings::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings = serde_json::from_str::<Settings>(&content).map_err(|e| e.to_string())?;
    Ok(settings)
}

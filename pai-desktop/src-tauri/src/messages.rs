use crate::{AppState, ChatMessage};
use std::fs;
use std::path::PathBuf;
use tauri::State;

pub fn get_messages_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
        .join("messages")
}

#[tauri::command]
pub fn get_messages(state: State<'_, AppState>) -> Result<Vec<ChatMessage>, String> {
    let messages = state.messages.lock().map_err(|e| e.to_string())?;
    Ok(messages.clone())
}

#[tauri::command]
pub fn add_message(state: State<'_, AppState>, message: ChatMessage) -> Result<(), String> {
    let messages_dir = get_messages_dir();
    fs::create_dir_all(&messages_dir).map_err(|e| e.to_string())?;

    let path = messages_dir.join(format!("{}.json", message.timestamp));
    let json = serde_json::to_string_pretty(&message).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    let mut messages = state.messages.lock().map_err(|e| e.to_string())?;
    messages.push(message);

    Ok(())
}

#[tauri::command]
pub fn clear_messages(state: State<'_, AppState>) -> Result<(), String> {
    let messages_dir = get_messages_dir();
    
    if messages_dir.exists() {
        fs::remove_dir_all(&messages_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&messages_dir).map_err(|e| e.to_string())?;
    }

    let mut messages = state.messages.lock().map_err(|e| e.to_string())?;
    messages.clear();

    Ok(())
}

pub fn load_messages_from_disk_sync() -> Vec<ChatMessage> {
    let messages_dir = get_messages_dir();
    
    if !messages_dir.exists() {
        return Vec::new();
    }

    let mut messages = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&messages_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(message) = serde_json::from_str::<ChatMessage>(&content) {
                        messages.push(message);
                    }
                }
            }
        }
    }

    messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    messages
}

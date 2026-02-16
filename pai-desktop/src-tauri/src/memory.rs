use crate::{AppState, MemoryItem};
use std::fs;
use std::path::PathBuf;
use tauri::State;

pub fn get_memory_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
        .join("memory")
}

#[tauri::command]
pub fn get_memories(state: State<'_, AppState>) -> Result<Vec<MemoryItem>, String> {
    let memories = state.memories.lock().map_err(|e| e.to_string())?;
    Ok(memories.clone())
}

#[tauri::command]
pub fn save_memory(state: State<'_, AppState>, memory: MemoryItem) -> Result<(), String> {
    let memory_dir = get_memory_dir();
    fs::create_dir_all(&memory_dir).map_err(|e| e.to_string())?;

    let path = memory_dir.join(format!("{}.json", memory.id));
    let json = serde_json::to_string_pretty(&memory).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    let mut memories = state.memories.lock().map_err(|e| e.to_string())?;
    memories.push(memory);

    Ok(())
}

#[tauri::command]
pub fn load_memories_from_disk(state: State<'_, AppState>) -> Result<Vec<MemoryItem>, String> {
    let memory_dir = get_memory_dir();
    
    if !memory_dir.exists() {
        return Ok(Vec::new());
    }

    let mut memories = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&memory_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(memory) = serde_json::from_str::<MemoryItem>(&content) {
                        memories.push(memory);
                    }
                }
            }
        }
    }

    memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut state_memories = state.memories.lock().map_err(|e| e.to_string())?;
    *state_memories = memories.clone();

    Ok(memories)
}

#[tauri::command]
pub fn delete_memory(state: State<'_, AppState>, memory_id: String) -> Result<(), String> {
    let memory_dir = get_memory_dir();
    let path = memory_dir.join(format!("{}.json", memory_id));
    
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    let mut memories = state.memories.lock().map_err(|e| e.to_string())?;
    memories.retain(|m| m.id != memory_id);

    Ok(())
}

pub fn load_memories_from_disk_sync() -> Vec<MemoryItem> {
    let memory_dir = get_memory_dir();
    
    if !memory_dir.exists() {
        return Vec::new();
    }

    let mut memories = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&memory_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(memory) = serde_json::from_str::<MemoryItem>(&content) {
                        memories.push(memory);
                    }
                }
            }
        }
    }

    memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    memories
}

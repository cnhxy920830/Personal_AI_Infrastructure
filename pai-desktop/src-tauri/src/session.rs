use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub last_active: i64,
    pub message_count: usize,
}

pub fn get_sessions_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
        .join("sessions")
}

pub fn get_current_session_file() -> PathBuf {
    get_sessions_dir().join("current_session")
}

pub fn ensure_sessions_dir() -> Result<(), String> {
    let dir = get_sessions_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_current_session() -> Result<Session, String> {
    ensure_sessions_dir()?;
    
    let current_file = get_current_session_file();
    
    if current_file.exists() {
        let content = fs::read_to_string(&current_file).map_err(|e| e.to_string())?;
        let session: Session = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(session);
    }
    
    create_new_session("Default".to_string())
}

#[tauri::command]
pub fn create_new_session(name: String) -> Result<Session, String> {
    ensure_sessions_dir()?;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;
    
    let session = Session {
        id: format!("session-{}", now),
        name,
        created_at: now,
        last_active: now,
        message_count: 0,
    };
    
    let sessions_dir = get_sessions_dir();
    let session_file = sessions_dir.join(format!("{}.json", session.id));
    
    let json = serde_json::to_string_pretty(&session).map_err(|e| e.to_string())?;
    fs::write(&session_file, &json).map_err(|e| e.to_string())?;
    
    let current_file = get_current_session_file();
    fs::write(&current_file, &json).map_err(|e| e.to_string())?;
    
    Ok(session)
}

pub fn update_session_activity(session: &mut Session) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;
    
    session.last_active = now;
    
    let sessions_dir = get_sessions_dir();
    let session_file = sessions_dir.join(format!("{}.json", session.id));
    
    let json = serde_json::to_string_pretty(&session).map_err(|e| e.to_string())?;
    fs::write(&session_file, &json).map_err(|e| e.to_string())?;
    
    let current_file = get_current_session_file();
    fs::write(&current_file, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn increment_message_count(session: &mut Session) -> Result<(), String> {
    session.message_count += 1;
    update_session_activity(session)
}

#[tauri::command]
pub fn list_sessions() -> Result<Vec<Session>, String> {
    ensure_sessions_dir()?;
    
    let sessions_dir = get_sessions_dir();
    let mut sessions = Vec::new();
    
    let entries = fs::read_dir(&sessions_dir).map_err(|e| e.to_string())?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("session-") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }
    }
    
    sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    
    Ok(sessions)
}

#[tauri::command]
pub fn switch_session(session_id: String) -> Result<Session, String> {
    ensure_sessions_dir()?;
    
    let sessions_dir = get_sessions_dir();
    let session_file = sessions_dir.join(format!("{}.json", session_id));
    
    if !session_file.exists() {
        return Err("Session not found".to_string());
    }
    
    let content = fs::read_to_string(&session_file).map_err(|e| e.to_string())?;
    let session: Session = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    let current_file = get_current_session_file();
    fs::write(&current_file, content).map_err(|e| e.to_string())?;
    
    Ok(session)
}

#[tauri::command]
pub fn delete_session(session_id: String) -> Result<(), String> {
    ensure_sessions_dir()?;
    
    let sessions_dir = get_sessions_dir();
    let session_file = sessions_dir.join(format!("{}.json", session_id));
    
    if !session_file.exists() {
        return Err("Session not found".to_string());
    }
    
    let current = get_current_session()?;
    if current.id == session_id {
        return Err("Cannot delete current session".to_string());
    }
    
    fs::remove_file(&session_file).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn rename_session(session_id: String, new_name: String) -> Result<Session, String> {
    ensure_sessions_dir()?;
    
    let sessions_dir = get_sessions_dir();
    let session_file = sessions_dir.join(format!("{}.json", session_id));
    
    if !session_file.exists() {
        return Err("Session not found".to_string());
    }
    
    let content = fs::read_to_string(&session_file).map_err(|e| e.to_string())?;
    let mut session: Session = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    session.name = new_name;
    
    let json = serde_json::to_string_pretty(&session).map_err(|e| e.to_string())?;
    fs::write(&session_file, &json).map_err(|e| e.to_string())?;
    
    let current_file = get_current_session_file();
    if let Ok(current) = get_current_session() {
        if current.id == session_id {
            fs::write(&current_file, &json).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(session)
}

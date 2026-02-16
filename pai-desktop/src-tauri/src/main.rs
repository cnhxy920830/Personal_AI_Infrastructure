#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod memory;
mod messages;
mod settings;
mod skills;

use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Default)]
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub memories: Mutex<Vec<MemoryItem>>,
    pub messages: Mutex<Vec<ChatMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub google_api_key: String,
    pub xai_api_key: String,
    pub perplexity_api_key: String,
    pub elevenlabs_api_key: String,
    pub default_model: String,
    pub voice_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            anthropic_api_key: String::new(),
            openai_api_key: String::new(),
            google_api_key: String::new(),
            xai_api_key: String::new(),
            perplexity_api_key: String::new(),
            elevenlabs_api_key: String::new(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            voice_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub memory_type: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting PAI Desktop...");

    let settings = settings::load_settings_from_disk().unwrap_or_default();
    let memories = memory::load_memories_from_disk_sync();
    let messages = messages::load_messages_from_disk_sync();

    info!("Loaded {} settings, {} memories, {} messages", 
        if settings.anthropic_api_key.is_empty() && settings.openai_api_key.is_empty() { 0 } else { 1 },
        memories.len(),
        messages.len()
    );

    let app_state = AppState {
        settings: Mutex::new(settings),
        memories: Mutex::new(memories),
        messages: Mutex::new(messages),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            ai::chat,
            settings::get_settings,
            settings::save_settings,
            skills::get_skills,
            memory::get_memories,
            memory::save_memory,
            memory::load_memories_from_disk,
            memory::delete_memory,
            messages::get_messages,
            messages::add_message,
            messages::clear_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

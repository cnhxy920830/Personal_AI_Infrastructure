#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod algorithm;
mod ai;
mod hooks;
mod memory;
mod messages;
mod settings;
mod session;
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
    pub tags: Vec<String>,
    pub entities: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNote {
    pub note_type: String,
    pub content: String,
    pub entity: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
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
            ai::get_models,
            settings::get_settings,
            settings::save_settings,
            skills::get_skills,
            skills::save_skill,
            skills::get_skill_content,
            skills::delete_skill,
            session::get_current_session,
            session::create_new_session,
            session::list_sessions,
            session::switch_session,
            session::delete_session,
            session::rename_session,
            memory::get_memories,
            memory::save_memory,
            memory::load_memories_from_disk,
            memory::delete_memory,
            memory::search_memories,
            memory::save_relationship_note,
            memory::get_relationship_notes,
            memory::save_work_item,
            memory::get_work_items,
            memory::complete_work_item,
            memory::save_prd,
            memory::get_prds,
            messages::get_messages,
            messages::add_message,
            messages::clear_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

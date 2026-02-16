use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub mod algorithm;
pub mod ai;
pub mod hooks;
pub mod memory;
pub mod messages;
pub mod session;
pub mod settings;
pub mod skills;

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

pub use ai::chat;
pub use settings::{get_settings, save_settings};
pub use skills::{get_skills, save_skill, get_skill_content, delete_skill};
pub use session::{get_current_session, create_new_session, list_sessions, switch_session, delete_session, rename_session};
pub use memory::{
    get_memories, save_memory, delete_memory, load_memories_from_disk,
    search_memories, save_relationship_note, get_relationship_notes,
    save_work_item, get_work_items, complete_work_item,
    save_prd, get_prds,
};

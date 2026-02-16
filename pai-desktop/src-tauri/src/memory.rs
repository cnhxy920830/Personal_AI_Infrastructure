use crate::{AppState, MemoryItem, RelationshipNote, WorkItem};
use std::fs;
use std::path::PathBuf;
use tauri::State;

pub fn save_memory_internal(memory: &MemoryItem) -> Result<(), String> {
    ensure_dirs()?;
    
    let memory_dir = match memory.memory_type.as_str() {
        "WORK" => get_work_dir(),
        "LEARNING" => get_learning_dir(),
        "RELATIONSHIP" => get_relationship_dir(),
        _ => get_memory_dir(),
    };

    let path = memory_dir.join(format!("{}.md", memory.id));
    let frontmatter = format!(
        "---\nid: {}\ntitle: {}\ntype: {}\ntags: {}\nentities: {}\nconfidence: {}\ntimestamp: {}\n---\n\n{}",
        memory.id,
        memory.title,
        memory.memory_type,
        memory.tags.join(", "),
        memory.entities.join(", "),
        memory.confidence,
        memory.timestamp,
        memory.content
    );
    fs::write(&path, frontmatter).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn get_base_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
}

pub fn get_memory_dir() -> PathBuf {
    get_base_dir().join("memory")
}

pub fn get_work_dir() -> PathBuf {
    get_base_dir().join("memory").join("WORK")
}

pub fn get_learning_dir() -> PathBuf {
    get_base_dir().join("memory").join("LEARNING")
}

pub fn get_relationship_dir() -> PathBuf {
    get_base_dir().join("memory").join("RELATIONSHIP")
}

pub fn get_prd_dir() -> PathBuf {
    get_base_dir().join("prd")
}

fn ensure_dirs() -> Result<(), String> {
    let dirs = [
        get_memory_dir(),
        get_work_dir(),
        get_learning_dir(),
        get_relationship_dir(),
        get_prd_dir(),
    ];
    for dir in dirs {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir {:?}: {}", dir, e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_memories(state: State<'_, AppState>) -> Result<Vec<MemoryItem>, String> {
    let memories = state.memories.lock().map_err(|e| e.to_string())?;
    Ok(memories.clone())
}

#[tauri::command]
pub fn save_memory(state: State<'_, AppState>, memory: MemoryItem) -> Result<(), String> {
    ensure_dirs()?;
    
    let memory_dir = match memory.memory_type.as_str() {
        "WORK" => get_work_dir(),
        "LEARNING" => get_learning_dir(),
        "RELATIONSHIP" => get_relationship_dir(),
        _ => get_memory_dir(),
    };

    let path = memory_dir.join(format!("{}.md", memory.id));
    let frontmatter = format!(
        "---\nid: {}\ntitle: {}\ntype: {}\ntags: {}\nentities: {}\nconfidence: {}\ntimestamp: {}\n---\n\n{}",
        memory.id,
        memory.title,
        memory.memory_type,
        memory.tags.join(", "),
        memory.entities.join(", "),
        memory.confidence,
        memory.timestamp,
        memory.content
    );
    fs::write(&path, frontmatter).map_err(|e| e.to_string())?;

    let mut memories = state.memories.lock().map_err(|e| e.to_string())?;
    memories.push(memory);

    Ok(())
}

#[tauri::command]
pub fn load_memories_from_disk(state: State<'_, AppState>) -> Result<Vec<MemoryItem>, String> {
    ensure_dirs()?;
    let mut all_memories = Vec::new();

    let dirs = [
        (get_work_dir(), "WORK"),
        (get_learning_dir(), "LEARNING"),
        (get_relationship_dir(), "RELATIONSHIP"),
        (get_memory_dir(), "general"),
    ];

    for (dir, mem_type) in dirs {
        if !dir.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(memory) = parse_markdown_memory(&content, mem_type) {
                            all_memories.push(memory);
                        }
                    }
                }
            }
        }
    }

    all_memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut state_memories = state.memories.lock().map_err(|e| e.to_string())?;
    *state_memories = all_memories.clone();

    Ok(all_memories)
}

fn parse_markdown_memory(content: &str, default_type: &str) -> Option<MemoryItem> {
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1];
    let body = parts[2].trim();

    let mut id = String::new();
    let mut title = String::new();
    let mut memory_type = default_type.to_string();
    let mut tags = Vec::new();
    let mut entities = Vec::new();
    let mut confidence = 1.0f32;
    let mut timestamp = 0i64;

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(": ") {
            let value = value.trim();
            match key {
                "id" => id = value.to_string(),
                "title" => title = value.to_string(),
                "type" => memory_type = value.to_string(),
                "tags" => tags = value.split(", ").map(|s| s.to_string()).collect(),
                "entities" => entities = value.split(", ").map(|s| s.to_string()).collect(),
                "confidence" => confidence = value.parse().unwrap_or(1.0),
                "timestamp" => timestamp = value.parse().unwrap_or(0),
                _ => {}
            }
        }
    }

    if id.is_empty() || title.is_empty() {
        return None;
    }

    Some(MemoryItem {
        id,
        title,
        content: body.to_string(),
        memory_type,
        timestamp,
        tags,
        entities,
        confidence,
    })
}

#[tauri::command]
pub fn delete_memory(state: State<'_, AppState>, memory_id: String) -> Result<(), String> {
    let dirs = [get_work_dir(), get_learning_dir(), get_relationship_dir(), get_memory_dir()];

    for dir in dirs {
        let path = dir.join(format!("{}.md", memory_id));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            break;
        }
    }

    let mut memories = state.memories.lock().map_err(|e| e.to_string())?;
    memories.retain(|m| m.id != memory_id);

    Ok(())
}

#[tauri::command]
pub fn search_memories(query: String, memory_type: Option<String>) -> Result<Vec<MemoryItem>, String> {
    let memories = load_memories_from_disk_internal()?;
    let query_lower = query.to_lowercase();

    let filtered: Vec<MemoryItem> = memories
        .into_iter()
        .filter(|m| {
            if let Some(ref t) = memory_type {
                if m.memory_type != *t {
                    return false;
                }
            }
            m.title.to_lowercase().contains(&query_lower)
                || m.content.to_lowercase().contains(&query_lower)
                || m.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect();

    Ok(filtered)
}

fn load_memories_from_disk_internal() -> Result<Vec<MemoryItem>, String> {
    ensure_dirs()?;
    let mut all_memories = Vec::new();

    let dirs = [
        (get_work_dir(), "WORK"),
        (get_learning_dir(), "LEARNING"),
        (get_relationship_dir(), "RELATIONSHIP"),
        (get_memory_dir(), "general"),
    ];

    for (dir, mem_type) in dirs {
        if !dir.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(memory) = parse_markdown_memory(&content, mem_type) {
                            all_memories.push(memory);
                        }
                    }
                }
            }
        }
    }

    all_memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(all_memories)
}

#[tauri::command]
pub fn save_relationship_note(note: RelationshipNote) -> Result<(), String> {
    ensure_dirs()?;
    
    let relationship_dir = get_relationship_dir();
    let timestamp = chrono::Utc::now().format("%Y-%m").to_string();
    let dir = relationship_dir.join(&timestamp);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filename = format!("{}.md", date);
    let path = dir.join(&filename);

    let content = format!(
        "## {} @{}\n\n{}\n\n---\n",
        note.note_type, note.entity, note.content
    );

    if path.exists() {
        let existing = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let new_content = format!("{}{}", existing, content);
        fs::write(&path, new_content).map_err(|e| e.to_string())?;
    } else {
        fs::write(&path, content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_relationship_notes() -> Result<Vec<RelationshipNote>, String> {
    ensure_dirs()?;
    let relationship_dir = get_relationship_dir();
    let mut notes = Vec::new();

    if !relationship_dir.exists() {
        return Ok(notes);
    }

    if let Ok(months) = fs::read_dir(&relationship_dir) {
        for month in months.flatten() {
            let month_path = month.path();
            if month_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&month_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "md") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                notes.extend(parse_relationship_notes(&content));
                            }
                        }
                    }
                }
            }
        }
    }

    notes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(notes)
}

fn parse_relationship_notes(content: &str) -> Vec<RelationshipNote> {
    let mut notes = Vec::new();
    let timestamp = chrono::Utc::now().timestamp();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("## ") {
            let parts: Vec<&str> = line[3..].splitn(2, " @").collect();
            if parts.len() == 2 {
                notes.push(RelationshipNote {
                    note_type: parts[0].to_string(),
                    content: String::new(),
                    entity: parts[1].to_string(),
                    timestamp,
                });
            }
        } else if !line.starts_with("---") && !line.is_empty() && !line.starts_with("##") {
            if let Some(last) = notes.last_mut() {
                if !last.content.is_empty() {
                    last.content.push_str("\n");
                }
                last.content.push_str(line);
            }
        }
    }

    notes
}

#[tauri::command]
pub fn save_work_item(work: WorkItem) -> Result<(), String> {
    ensure_dirs()?;
    
    let work_dir = get_work_dir();
    let work_subdir = work_dir.join(&work.id);
    fs::create_dir_all(&work_subdir).map_err(|e| e.to_string())?;

    let meta_path = work_subdir.join("META.yaml");
    let meta_content = format!(
        "id: {}\ntitle: {}\nstatus: {}\ncreated_at: {}\ncompleted_at: {}\n",
        work.id,
        work.title,
        work.status,
        work.created_at,
        work.completed_at.map(|t| t.to_string()).unwrap_or_default()
    );
    fs::write(&meta_path, meta_content).map_err(|e| e.to_string())?;

    let desc_path = work_subdir.join("description.md");
    fs::write(&desc_path, &work.description).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_work_items() -> Result<Vec<WorkItem>, String> {
    ensure_dirs()?;
    let work_dir = get_work_dir();
    let mut items = Vec::new();

    if !work_dir.exists() {
        return Ok(items);
    }

    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let meta_path = path.join("META.yaml");
                if let Ok(content) = fs::read_to_string(&meta_path) {
                    if let Some(work) = parse_work_meta(&content, path.file_name().unwrap().to_str().unwrap()) {
                        items.push(work);
                    }
                }
            }
        }
    }

    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

fn parse_work_meta(content: &str, id: &str) -> Option<WorkItem> {
    let mut title = String::new();
    let mut status = "active".to_string();
    let mut created_at = 0i64;
    let mut completed_at = None;

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(": ") {
            let value = value.trim();
            match key {
                "title" => title = value.to_string(),
                "status" => status = value.to_string(),
                "created_at" => created_at = value.parse().unwrap_or(0),
                "completed_at" => {
                    if !value.is_empty() {
                        completed_at = value.parse().ok();
                    }
                }
                _ => {}
            }
        }
    }

    if title.is_empty() {
        return None;
    }

    Some(WorkItem {
        id: id.to_string(),
        title,
        description: String::new(),
        status,
        created_at,
        completed_at,
    })
}

#[tauri::command]
pub fn complete_work_item(work_id: String) -> Result<(), String> {
    let work_dir = get_work_dir().join(&work_id);
    let meta_path = work_dir.join("META.yaml");

    if !meta_path.exists() {
        return Err(format!("Work item {} not found", work_id));
    }

    let content = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
    let completed_at = chrono::Utc::now().timestamp();
    
    let new_content = format!(
        "{}\ncompleted_at: {}\nstatus: COMPLETED",
        content.trim(),
        completed_at
    );
    
    fs::write(&meta_path, new_content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_prd(prd_id: String, content: String) -> Result<(), String> {
    ensure_dirs()?;
    
    let prd_dir = get_prd_dir();
    let path = prd_dir.join(format!("{}.md", prd_id));
    fs::write(&path, content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_prds() -> Result<Vec<(String, String)>, String> {
    ensure_dirs()?;
    let prd_dir = get_prd_dir();
    let mut prds = Vec::new();

    if !prd_dir.exists() {
        return Ok(prds);
    }

    if let Ok(entries) = fs::read_dir(&prd_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let id = path.file_stem().unwrap().to_str().unwrap().to_string();
                    prds.push((id, content));
                }
            }
        }
    }

    Ok(prds)
}

pub fn load_memories_from_disk_sync() -> Vec<MemoryItem> {
    load_memories_from_disk_internal().unwrap_or_default()
}

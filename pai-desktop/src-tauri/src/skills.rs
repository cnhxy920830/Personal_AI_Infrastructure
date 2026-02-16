use crate::Skill;
use std::fs;
use std::path::PathBuf;

pub fn get_skills_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PAI")
        .join("skills")
}

pub fn get_all_skills() -> Vec<Skill> {
    let mut skills = get_builtin_skills();
    
    if let Ok(custom) = get_custom_skills() {
        skills.extend(custom);
    }
    
    skills
}

pub fn get_builtin_skills() -> Vec<Skill> {
    vec![
        Skill {
            id: "agents".to_string(),
            name: "Agents".to_string(),
            description: "Dynamic agent composition and management system".to_string(),
            category: "core".to_string(),
        },
        Skill {
            id: "research".to_string(),
            name: "Research".to_string(),
            description: "Comprehensive research, analysis and content extraction".to_string(),
            category: "core".to_string(),
        },
        Skill {
            id: "telos".to_string(),
            name: "Telos".to_string(),
            description: "Life OS and project analysis framework".to_string(),
            category: "core".to_string(),
        },
        Skill {
            id: "redteam".to_string(),
            name: "RedTeam".to_string(),
            description: "Security assessment and red team operations".to_string(),
            category: "security".to_string(),
        },
        Skill {
            id: "recon".to_string(),
            name: "Recon".to_string(),
            description: "Information gathering and reconnaissance".to_string(),
            category: "security".to_string(),
        },
        Skill {
            id: "osint".to_string(),
            name: "OSINT".to_string(),
            description: "Open source intelligence".to_string(),
            category: "security".to_string(),
        },
        Skill {
            id: "browser".to_string(),
            name: "Browser".to_string(),
            description: "Browser automation and control".to_string(),
            category: "tools".to_string(),
        },
        Skill {
            id: "art".to_string(),
            name: "Art".to_string(),
            description: "Art generation and creative tools".to_string(),
            category: "creative".to_string(),
        },
        Skill {
            id: "documents".to_string(),
            name: "Documents".to_string(),
            description: "Document processing (PDF, Docx, Xlsx, Pptx)".to_string(),
            category: "tools".to_string(),
        },
        Skill {
            id: "apify".to_string(),
            name: "Apify".to_string(),
            description: "Web scraping and automation".to_string(),
            category: "tools".to_string(),
        },
        Skill {
            id: "prompting".to_string(),
            name: "Prompting".to_string(),
            description: "Prompt engineering and optimization".to_string(),
            category: "ai".to_string(),
        },
        Skill {
            id: "fabric".to_string(),
            name: "Fabric".to_string(),
            description: "AI patterns library (242+ patterns)".to_string(),
            category: "ai".to_string(),
        },
        Skill {
            id: "evals".to_string(),
            name: "Evals".to_string(),
            description: "Evaluation and testing framework".to_string(),
            category: "ai".to_string(),
        },
        Skill {
            id: "council".to_string(),
            name: "Council".to_string(),
            description: "Multi-agent decision committee".to_string(),
            category: "ai".to_string(),
        },
        Skill {
            id: "firstprinciples".to_string(),
            name: "First Principles".to_string(),
            description: "First principles thinking and analysis".to_string(),
            category: "ai".to_string(),
        },
        Skill {
            id: "becreative".to_string(),
            name: "BeCreative".to_string(),
            description: "Creative brainstorming and ideation".to_string(),
            category: "creative".to_string(),
        },
        Skill {
            id: "paiupgrade".to_string(),
            name: "PAI Upgrade".to_string(),
            description: "Auto upgrade system for PAI".to_string(),
            category: "system".to_string(),
        },
        Skill {
            id: "createskill".to_string(),
            name: "CreateSkill".to_string(),
            description: "Tool for creating custom skills".to_string(),
            category: "tools".to_string(),
        },
        Skill {
            id: "createcli".to_string(),
            name: "CreateCLI".to_string(),
            description: "Tool for creating CLI applications".to_string(),
            category: "tools".to_string(),
        },
        Skill {
            id: "extractwisdom".to_string(),
            name: "Extract Wisdom".to_string(),
            description: "Extract insights and wisdom from content".to_string(),
            category: "ai".to_string(),
        },
    ]
}

fn get_custom_skills() -> Result<Vec<Skill>, String> {
    let skills_dir = get_skills_dir();
    
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();
    
    let entries = fs::read_dir(&skills_dir).map_err(|e| e.to_string())?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") || path.extension().map_or(false, |ext| ext == "yaml") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(skill) = parse_skill_file(&path, &content) {
                    skills.push(skill);
                }
            }
        }
    }

    Ok(skills)
}

fn parse_skill_file(path: &PathBuf, content: &str) -> Option<Skill> {
    let id = path.file_stem()?.to_str()?.to_string();
    
    let mut name = id.clone();
    let mut description = String::new();
    let mut category = "custom".to_string();
    
    if content.starts_with("---") {
        if let Some(end) = content.find("---") {
            let frontmatter = &content[3..end];
            for line in frontmatter.lines() {
                let line = line.trim();
                if line.starts_with("name:") {
                    name = line[5..].trim().to_string();
                } else if line.starts_with("description:") {
                    description = line[12..].trim().to_string();
                } else if line.starts_with("category:") {
                    category = line[9..].trim().to_string();
                }
            }
        }
    } else {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("# ") {
                name = first_line[2..].trim().to_string();
            }
        }
        description = content.lines().skip(1).take(2).collect::<Vec<_>>().join(" ");
    }

    Some(Skill {
        id,
        name,
        description,
        category,
    })
}

#[tauri::command]
pub fn get_skills() -> Vec<Skill> {
    get_all_skills()
}

#[tauri::command]
pub fn save_skill(id: String, name: String, description: String, category: String, content: String) -> Result<(), String> {
    let skills_dir = get_skills_dir();
    fs::create_dir_all(&skills_dir).map_err(|e| e.to_string())?;

    let skill_content = format!(
        "---\nname: {}\ndescription: {}\ncategory: {}\n---\n\n{}",
        name, description, category, content
    );

    let path = skills_dir.join(format!("{}.md", id));
    fs::write(&path, skill_content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_skill_content(id: String) -> Result<String, String> {
    let skills_dir = get_skills_dir();
    let path = skills_dir.join(format!("{}.md", id));
    
    if path.exists() {
        fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Err("Skill not found".to_string())
    }
}

#[tauri::command]
pub fn delete_skill(id: String) -> Result<(), String> {
    let skills_dir = get_skills_dir();
    let path = skills_dir.join(format!("{}.md", id));
    
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Err("Skill not found".to_string())
    }
}

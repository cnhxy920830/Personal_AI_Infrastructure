use crate::Skill;

pub fn get_all_skills() -> Vec<Skill> {
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

#[tauri::command]
pub fn get_skills() -> Vec<Skill> {
    get_all_skills()
}

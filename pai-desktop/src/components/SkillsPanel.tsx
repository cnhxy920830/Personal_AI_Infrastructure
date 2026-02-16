import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore } from "../store/useI18nStore";

interface Skill {
  id: string;
  name: string;
  description: string;
  category: string;
}

export function SkillsPanel() {
  const { t } = useI18nStore();
  const [skills, setSkills] = useState<Skill[]>([]);
  const [selectedSkill, setSelectedSkill] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    try {
      const skillsData = await invoke<Skill[]>("get_skills");
      setSkills(skillsData);
    } catch (error) {
      console.error("Failed to load skills:", error);
      setSkills(getDefaultSkills());
    } finally {
      setLoading(false);
    }
  };

  const getDefaultSkills = (): Skill[] => [
    { id: "agents", name: "Agents", description: "Dynamic agent composition", category: "core" },
    { id: "research", name: "Research", description: "Comprehensive research system", category: "core" },
    { id: "telos", name: "Telos", description: "Life OS and project analysis", category: "core" },
    { id: "redteam", name: "RedTeam", description: "Security assessment", category: "security" },
    { id: "recon", name: "Recon", description: "Information gathering", category: "security" },
    { id: "osint", name: "OSINT", description: "Open source intelligence", category: "security" },
    { id: "browser", name: "Browser", description: "Browser automation", category: "tools" },
    { id: "art", name: "Art", description: "Art and image generation", category: "creative" },
    { id: "documents", name: "Documents", description: "Document processing", category: "tools" },
    { id: "apify", name: "Apify", description: "Web scraping", category: "tools" },
    { id: "prompting", name: "Prompting", description: "Prompt engineering", category: "ai" },
    { id: "fabric", name: "Fabric", description: "AI patterns library", category: "ai" },
    { id: "evals", name: "Evals", description: "Evaluation system", category: "ai" },
    { id: "council", name: "Council", description: "Decision committee", category: "ai" },
    { id: "firstprinciples", name: "First Principles", description: "First principles thinking", category: "ai" },
    { id: "becreative", name: "BeCreative", description: "Creative brainstorming", category: "creative" },
    { id: "paiupgrade", name: "PAI Upgrade", description: "Auto upgrade system", category: "system" },
    { id: "createskill", name: "CreateSkill", description: "Skill creation tool", category: "tools" },
    { id: "createcli", name: "CreateCLI", description: "CLI creation tool", category: "tools" },
  ];

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.skills.title}</h2>
      </div>
      <div class="skills-grid">
        {loading ? (
          <div>{t.skills.loading}</div>
        ) : (
          skills.map((skill) => (
            <div
              key={skill.id}
              class={`skill-card ${selectedSkill === skill.id ? "selected" : ""}`}
              onClick={() => setSelectedSkill(skill.id)}
            >
              <div class="skill-name">{skill.name}</div>
              <div class="skill-description">{skill.description}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

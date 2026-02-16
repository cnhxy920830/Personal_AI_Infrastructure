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
  const [selectedSkill, setSelectedSkill] = useState<Skill | null>(null);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newSkill, setNewSkill] = useState({ id: "", name: "", description: "", category: "custom", content: "" });
  const [skillContent, setSkillContent] = useState("");

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    try {
      const skillsData = await invoke<Skill[]>("get_skills");
      setSkills(skillsData);
    } catch (error) {
      console.error("Failed to load skills:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSkillClick = async (skill: Skill) => {
    setSelectedSkill(skill);
    try {
      const content = await invoke<string>("get_skill_content", { id: skill.id });
      setSkillContent(content);
    } catch (error) {
      setSkillContent(skill.description);
    }
  };

  const handleAddSkill = async () => {
    if (!newSkill.id || !newSkill.name) return;
    try {
      await invoke("save_skill", {
        id: newSkill.id.toLowerCase().replace(/\s+/g, "-"),
        name: newSkill.name,
        description: newSkill.description,
        category: newSkill.category,
        content: newSkill.content || newSkill.description,
      });
      setShowAddForm(false);
      setNewSkill({ id: "", name: "", description: "", category: "custom", content: "" });
      loadSkills();
    } catch (error) {
      console.error("Failed to save skill:", error);
    }
  };

  const handleDeleteSkill = async (skillId: string) => {
    try {
      await invoke("delete_skill", { id: skillId });
      if (selectedSkill?.id === skillId) {
        setSelectedSkill(null);
        setSkillContent("");
      }
      loadSkills();
    } catch (error) {
      console.error("Failed to delete skill:", error);
    }
  };

  const categories = [...new Set(skills.map(s => s.category))];
  const builtinCategories = ["core", "security", "tools", "ai", "creative", "system"];

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.skills.title}</h2>
        <button class="btn-add" onClick={() => setShowAddForm(!showAddForm)}>
          {showAddForm ? "✕" : "+"}
        </button>
      </div>

      {showAddForm && (
        <div class="skill-form">
          <input
            type="text"
            placeholder={t.skills.formName || "Skill Name"}
            value={newSkill.name}
            onInput={(e) => setNewSkill({...newSkill, name: (e.target as HTMLInputElement).value})}
            class="input"
          />
          <input
            type="text"
            placeholder="ID (e.g., my-skill)"
            value={newSkill.id}
            onInput={(e) => setNewSkill({...newSkill, id: (e.target as HTMLInputElement).value})}
            class="input"
          />
          <input
            type="text"
            placeholder={t.skills.formDesc || "Description"}
            value={newSkill.description}
            onInput={(e) => setNewSkill({...newSkill, description: (e.target as HTMLInputElement).value})}
            class="input"
          />
          <select
            value={newSkill.category}
            onChange={(e) => setNewSkill({...newSkill, category: (e.target as HTMLSelectElement).value})}
            class="select"
          >
            <option value="custom">Custom</option>
            <option value="core">Core</option>
            <option value="ai">AI</option>
            <option value="tools">Tools</option>
            <option value="creative">Creative</option>
            <option value="security">Security</option>
          </select>
          <textarea
            placeholder={t.skills.formContent || "Skill Content (Markdown)"}
            value={newSkill.content}
            onInput={(e) => setNewSkill({...newSkill, content: (e.target as HTMLTextAreaElement).value})}
            class="textarea"
          />
          <button class="btn-save" onClick={handleAddSkill}>
            {t.skills.save || "Save"}
          </button>
        </div>
      )}

      {selectedSkill && (
        <div class="skill-detail">
          <div class="skill-detail-header">
            <div>
              <h3>{selectedSkill.name}</h3>
              <span class={`skill-category ${selectedSkill.category}`}>{selectedSkill.category}</span>
            </div>
            {!builtinCategories.includes(selectedSkill.category) && (
              <button class="btn-delete" onClick={() => handleDeleteSkill(selectedSkill.id)}>
                {t.skills.delete || "Delete"}
              </button>
            )}
          </div>
          <pre class="skill-content">{skillContent}</pre>
        </div>
      )}

      <div class="skills-list">
        {loading ? (
          <div>{t.skills.loading}</div>
        ) : categories.map(category => (
          <div key={category} class="skill-category-group">
            <h4 class="category-title">{category}</h4>
            <div class="skills-grid">
              {skills.filter(s => s.category === category).map(skill => (
                <div
                  key={skill.id}
                  class={`skill-card ${selectedSkill?.id === skill.id ? "selected" : ""}`}
                  onClick={() => handleSkillClick(skill)}
                >
                  <div class="skill-name">{skill.name}</div>
                  <div class="skill-description">{skill.description}</div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

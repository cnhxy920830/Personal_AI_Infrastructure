import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore } from "../store/useI18nStore";

interface MemoryItem {
  id: string;
  title: string;
  content: string;
  memory_type: string;
  timestamp: number;
  tags: string[];
  entities: string[];
  confidence: number;
}

export function MemoryPanel() {
  const { t } = useI18nStore();
  const [memories, setMemories] = useState<MemoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterType, setFilterType] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [newMemory, setNewMemory] = useState({
    title: "",
    content: "",
    memory_type: "general",
    tags: "",
  });

  useEffect(() => {
    loadMemories();
  }, []);

  const loadMemories = async () => {
    try {
      const memoryData = await invoke<MemoryItem[]>("get_memories");
      setMemories(memoryData);
    } catch (error) {
      console.error("Failed to load memories:", error);
      setMemories([]);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      loadMemories();
      return;
    }
    try {
      const results = await invoke<MemoryItem[]>("search_memories", {
        query: searchQuery,
        memoryType: filterType === "all" ? null : filterType,
      });
      setMemories(results);
    } catch (error) {
      console.error("Search failed:", error);
    }
  };

  const handleSaveMemory = async () => {
    if (!newMemory.title.trim() || !newMemory.content.trim()) return;
    
    const memory: MemoryItem = {
      id: `memory-${Date.now()}`,
      title: newMemory.title,
      content: newMemory.content,
      memory_type: newMemory.memory_type,
      timestamp: Date.now(),
      tags: newMemory.tags.split(",").map(t => t.trim()).filter(t => t),
      entities: [],
      confidence: 1.0,
    };

    try {
      await invoke("save_memory", { memory });
      setShowAddForm(false);
      setNewMemory({ title: "", content: "", memory_type: "general", tags: "" });
      loadMemories();
    } catch (error) {
      console.error("Failed to save memory:", error);
    }
  };

  const handleDeleteMemory = async (id: string) => {
    try {
      await invoke("delete_memory", { memoryId: id });
      loadMemories();
    } catch (error) {
      console.error("Failed to delete memory:", error);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString();
  };

  const filteredMemories = filterType === "all" 
    ? memories 
    : memories.filter(m => m.memory_type === filterType);

  const memoryTypes = ["all", "WORK", "LEARNING", "RELATIONSHIP", "general"];

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.memory.title}</h2>
        <button class="btn-add" onClick={() => setShowAddForm(!showAddForm)}>
          {showAddForm ? "✕" : "+"}
        </button>
      </div>

      {showAddForm && (
        <div class="memory-form">
          <input
            type="text"
            placeholder={t.memory.formTitle || "Title"}
            value={newMemory.title}
            onInput={(e) => setNewMemory({...newMemory, title: (e.target as HTMLInputElement).value})}
            class="input"
          />
          <textarea
            placeholder={t.memory.formContent || "Content"}
            value={newMemory.content}
            onInput={(e) => setNewMemory({...newMemory, content: (e.target as HTMLTextAreaElement).value})}
            class="textarea"
          />
          <select
            value={newMemory.memory_type}
            onChange={(e) => setNewMemory({...newMemory, memory_type: (e.target as HTMLSelectElement).value})}
            class="select"
          >
            <option value="general">General</option>
            <option value="WORK">Work</option>
            <option value="LEARNING">Learning</option>
            <option value="RELATIONSHIP">Relationship</option>
          </select>
          <input
            type="text"
            placeholder="Tags (comma separated)"
            value={newMemory.tags}
            onInput={(e) => setNewMemory({...newMemory, tags: (e.target as HTMLInputElement).value})}
            class="input"
          />
          <button class="btn-save" onClick={handleSaveMemory}>
            {t.memory.save || "Save"}
          </button>
        </div>
      )}

      <div class="memory-filters">
        <input
          type="text"
          placeholder={t.memory.search || "Search..."}
          value={searchQuery}
          onInput={(e) => setSearchQuery((e.target as HTMLInputElement).value)}
          onKeyPress={(e) => e.key === "Enter" && handleSearch()}
          class="search-input"
        />
        <select
          value={filterType}
          onChange={(e) => setFilterType((e.target as HTMLSelectElement).value)}
          class="filter-select"
        >
          {memoryTypes.map(type => (
            <option key={type} value={type}>
              {type === "all" ? "All" : type}
            </option>
          ))}
        </select>
      </div>

      <div class="memory-list">
        {loading ? (
          <div>{t.memory.loading}</div>
        ) : filteredMemories.length === 0 ? (
          <div class="empty-state">
            <p>{t.memory.empty}</p>
          </div>
        ) : (
          filteredMemories.map((memory) => (
            <div key={memory.id} class="memory-item">
              <div class="memory-content">
                <div class="memory-header">
                  <span class={`memory-type-badge ${memory.memory_type.toLowerCase()}`}>
                    {memory.memory_type}
                  </span>
                  <span class="memory-date">{formatDate(memory.timestamp)}</span>
                </div>
                <div class="memory-title">{memory.title}</div>
                <div class="memory-text">{memory.content}</div>
                {memory.tags.length > 0 && (
                  <div class="memory-tags">
                    {memory.tags.map(tag => (
                      <span key={tag} class="memory-tag">{tag}</span>
                    ))}
                  </div>
                )}
              </div>
              <button 
                class="btn-delete" 
                onClick={() => handleDeleteMemory(memory.id)}
                title="Delete"
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

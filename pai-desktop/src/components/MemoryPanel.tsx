import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore } from "../store/useI18nStore";

interface MemoryItem {
  id: string;
  title: string;
  type: string;
  timestamp: number;
}

export function MemoryPanel() {
  const { t } = useI18nStore();
  const [memories, setMemories] = useState<MemoryItem[]>([]);
  const [loading, setLoading] = useState(true);

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

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString();
  };

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.memory.title}</h2>
      </div>
      <div class="memory-list">
        {loading ? (
          <div>{t.memory.loading}</div>
        ) : memories.length === 0 ? (
          <div class="empty-state">
            <p>{t.memory.empty}</p>
          </div>
        ) : (
          memories.map((memory) => (
            <div key={memory.id} class="memory-item">
              <div>
                <div class="memory-title">{memory.title}</div>
                <div class="memory-type">{memory.type}</div>
              </div>
              <div class="memory-date">{formatDate(memory.timestamp)}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

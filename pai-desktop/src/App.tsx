import { useState } from "preact/hooks";
import { ChatPanel } from "./components/ChatPanel";
import { Sidebar } from "./components/Sidebar";
import { SkillsPanel } from "./components/SkillsPanel";
import { MemoryPanel } from "./components/MemoryPanel";
import { SettingsPanel } from "./components/SettingsPanel";

export function App() {
  const [activePanel, setActivePanel] = useState<string>("chat");
  const [isConnected, setIsConnected] = useState(false);

  const renderPanel = () => {
    switch (activePanel) {
      case "chat":
        return <ChatPanel isConnected={isConnected} setIsConnected={setIsConnected} />;
      case "skills":
        return <SkillsPanel />;
      case "memory":
        return <MemoryPanel />;
      case "settings":
        return <SettingsPanel />;
      default:
        return <ChatPanel isConnected={isConnected} setIsConnected={setIsConnected} />;
    }
  };

  return (
    <div class="app-container">
      <Sidebar activePanel={activePanel} setActivePanel={setActivePanel} />
      <main class="main-content">{renderPanel()}</main>
    </div>
  );
}

import { useState, useRef, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore } from "../store/useI18nStore";

interface Message {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
}

interface ChatPanelProps {
  isConnected: boolean;
  setIsConnected: (connected: boolean) => void;
}

export function ChatPanel({ isConnected, setIsConnected }: ChatPanelProps) {
  const { t } = useI18nStore();
  
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const loadHistory = async () => {
    try {
      const history = await invoke<Message[]>("get_messages");
      if (history.length > 0) {
        setMessages(history);
      } else {
        setMessages([
          {
            role: "system",
            content: t.chat.welcome,
            timestamp: Date.now(),
          },
        ]);
      }
    } catch (error) {
      setMessages([
        {
          role: "system",
          content: t.chat.welcome,
          timestamp: Date.now(),
        },
      ]);
    }
  };

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      role: "user",
      content: input.trim(),
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);

    try {
      const response = await invoke<string>("chat", { message: userMessage.content });
      
      const assistantMessage: Message = {
        role: "assistant",
        content: response,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
      setIsConnected(true);
    } catch (error) {
      const errorMessage: Message = {
        role: "system",
        content: `${t.chat.error}: ${error}`,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.chat.title}</h2>
        <div class="connection-status">
          <span class={`status-dot ${isConnected ? "connected" : ""}`}></span>
          <span>{isConnected ? t.chat.connected : t.chat.disconnected}</span>
        </div>
      </div>
      <div class="chat-messages">
        {messages.map((msg, index) => (
          <div key={index} class={`message ${msg.role}`}>
            {msg.content}
          </div>
        ))}
        {isLoading && (
          <div class="message assistant">
            <span class="typing-indicator">{t.chat.thinking}</span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <div class="chat-input-container">
        <input
          type="text"
          class="chat-input"
          placeholder={t.chat.placeholder}
          value={input}
          onInput={(e) => setInput((e.target as HTMLInputElement).value)}
          onKeyPress={handleKeyPress}
          disabled={isLoading}
        />
        <button
          class="send-button"
          onClick={handleSend}
          disabled={isLoading || !input.trim()}
        >
          {t.chat.send}
        </button>
      </div>
    </div>
  );
}

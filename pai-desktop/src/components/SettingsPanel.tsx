import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore } from "../store/useI18nStore";
import { Locale } from "../i18n";

interface Settings {
  anthropic_api_key: string;
  openai_api_key: string;
  google_api_key: string;
  xai_api_key: string;
  perplexity_api_key: string;
  elevenlabs_api_key: string;
  default_model: string;
  voice_enabled: boolean;
}

interface Provider {
  id: string;
  name: string;
  key: keyof Settings;
}

export function SettingsPanel() {
  const { t, locale, setLocale } = useI18nStore();
  
  const [settings, setSettings] = useState<Settings>({
    anthropic_api_key: "",
    openai_api_key: "",
    google_api_key: "",
    xai_api_key: "",
    perplexity_api_key: "",
    elevenlabs_api_key: "",
    default_model: "claude-sonnet-4-20250514",
    voice_enabled: false,
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [showModal, setShowModal] = useState(false);
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null);
  const [tempApiKey, setTempApiKey] = useState("");
  const [showAddMenu, setShowAddMenu] = useState(false);

  const allProviders: Provider[] = [
    { id: "anthropic", name: "Anthropic (Claude)", key: "anthropic_api_key" },
    { id: "openai", name: "OpenAI (GPT)", key: "openai_api_key" },
    { id: "google", name: "Google (Gemini)", key: "google_api_key" },
    { id: "xai", name: "xAI (Grok)", key: "xai_api_key" },
    { id: "perplexity", name: "Perplexity", key: "perplexity_api_key" },
    { id: "elevenlabs", name: "ElevenLabs (Voice)", key: "elevenlabs_api_key" },
  ];

  const configuredProviders = allProviders.filter(p => !!settings[p.key]);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const savedSettings = await invoke<Settings>("get_settings");
      setSettings(savedSettings);
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("save_settings", { settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (error) {
      console.error("Failed to save settings:", error);
    } finally {
      setSaving(false);
    }
  };

  const updateSetting = (key: keyof Settings, value: string | boolean) => {
    setSettings((prev) => {
      const newSettings = { ...prev };
      (newSettings as any)[key] = value;
      return newSettings;
    });
  };

  const handleLanguageChange = (newLocale: Locale) => {
    setLocale(newLocale);
  };

  const openAddProviderModal = (provider: Provider) => {
    setEditingProvider(provider);
    const keyValue = settings[provider.key];
    setTempApiKey(typeof keyValue === 'string' ? keyValue : "");
    setShowModal(true);
    setShowAddMenu(false);
  };

  const saveProviderApiKey = () => {
    if (editingProvider) {
      updateSetting(editingProvider.key, tempApiKey);
    }
    setShowModal(false);
    setEditingProvider(null);
    setTempApiKey("");
  };

  const closeModal = () => {
    setShowModal(false);
    setEditingProvider(null);
    setTempApiKey("");
  };

  const getPlaceholder = (providerId: string) => {
    switch(providerId) {
      case 'anthropic': return 'sk-ant-...';
      case 'openai': return 'sk-...';
      case 'google': return 'AIza...';
      case 'xai': return 'xai-...';
      case 'perplexity': return 'pplx-...';
      case 'elevenlabs': return 'sk_...';
      default: return '';
    }
  };

  const unconfiguredProviders = allProviders.filter(p => !settings[p.key]);

  return (
    <div class="panel">
      <div class="panel-header">
        <h2 class="panel-title">{t.settings.title}</h2>
        <button
          class="send-button"
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? t.settings.saving : saved ? t.settings.saved : t.settings.save}
        </button>
      </div>
      <div class="settings-content">
        <div class="settings-section">
          <h3 class="settings-title">{t.settings.aiProviders}</h3>
          
          {configuredProviders.map((provider) => (
            <div 
              key={provider.id} 
              class="api-provider-item configured"
              onClick={() => openAddProviderModal(provider)}
            >
              <div>
                <div class="api-provider-name">{provider.name}</div>
                <div class="api-provider-status configured">
                  {t.settings.configured}
                </div>
              </div>
            </div>
          ))}
          
          {unconfiguredProviders.length > 0 && (
            <div style={{ position: 'relative' }}>
              <button 
                class="add-provider-btn" 
                onClick={() => setShowAddMenu(!showAddMenu)}
              >
                + {t.settings.addProvider}
              </button>
              
              {showAddMenu && (
                <div style={{
                  position: 'absolute',
                  top: '100%',
                  left: 0,
                  right: 0,
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: '8px',
                  marginTop: '4px',
                  zIndex: 10,
                  maxHeight: '200px',
                  overflowY: 'auto'
                }}>
                  {unconfiguredProviders.map(provider => (
                    <div
                      key={provider.id}
                      style={{
                        padding: '10px 12px',
                        cursor: 'pointer',
                        borderBottom: '1px solid var(--border)'
                      }}
                      onClick={() => openAddProviderModal(provider)}
                    >
                      {provider.name}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <div class="settings-section">
          <h3 class="settings-title">{t.settings.model}</h3>
          
          <div class="settings-item">
            <span class="settings-label">{t.settings.defaultModel}</span>
            <select
              class="settings-input"
              value={settings.default_model}
              onChange={(e) => updateSetting("default_model", (e.target as HTMLSelectElement).value)}
            >
              <option value="claude-opus-4-20250514">Claude Opus 4</option>
              <option value="claude-sonnet-4-20250514">Claude Sonnet 4</option>
              <option value="claude-haiku-3-20240307">Claude Haiku 3</option>
              <option value="gpt-4o">GPT-4o</option>
              <option value="gpt-4o-mini">GPT-4o Mini</option>
              <option value="o1">OpenAI o1</option>
              <option value="o1-mini">OpenAI o1-mini</option>
              <option value="o3-mini">OpenAI o3-mini</option>
              <option value="gemini-2.0-flash">Gemini 2.0 Flash</option>
              <option value="gemini-2.0-flash-lite">Gemini 2.0 Flash Lite</option>
              <option value="grok-2">Grok 2</option>
              <option value="grok-2-vision">Grok 2 Vision</option>
              <option value="perplexity-llama-3.1-sonar-large-128k-online">Perplexity Llama 3.1</option>
            </select>
          </div>
        </div>

        <div class="settings-section">
          <h3 class="settings-title">{t.settings.language}</h3>
          
          <div class="settings-item">
            <span class="settings-label">{t.settings.language}</span>
            <select
              class="settings-input"
              value={locale}
              onChange={(e) => handleLanguageChange((e.target as HTMLSelectElement).value as Locale)}
            >
              <option value="en">{t.settings.english}</option>
              <option value="zh">{t.settings.chinese}</option>
            </select>
          </div>
        </div>
      </div>

      {showModal && editingProvider && (
        <div class="modal-overlay" onClick={closeModal}>
          <div class="modal-content" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <h3 class="modal-title">{editingProvider.name}</h3>
              <button class="modal-close" onClick={closeModal}>&times;</button>
            </div>
            <div class="modal-section">
              <label class="modal-section-label">
                {t.settings.enterApiKey}
              </label>
              <input
                type="password"
                class="modal-section-input"
                placeholder={getPlaceholder(editingProvider.id)}
                value={tempApiKey}
                onInput={(e) => setTempApiKey((e.target as HTMLInputElement).value)}
              />
            </div>
            <div class="modal-actions">
              <button class="modal-cancel-btn" onClick={closeModal}>
                {t.settings.cancel}
              </button>
              <button class="modal-save-btn" onClick={saveProviderApiKey}>
                {t.settings.save}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

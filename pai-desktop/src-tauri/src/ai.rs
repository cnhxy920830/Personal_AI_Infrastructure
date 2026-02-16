use crate::{AppState, ChatMessage};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    #[serde(rename = "prompt_tokens")]
    prompt_tokens: u32,
    #[serde(rename = "completion_tokens")]
    completion_tokens: u32,
}

fn get_model_provider(model: &str) -> &'static str {
    if model.starts_with("claude-") {
        "anthropic"
    } else if model.starts_with("gpt-") || model.starts_with("o1") || model.starts_with("o3") {
        "openai"
    } else if model.starts_with("gemini-") {
        "google"
    } else if model.starts_with("grok-") {
        "xai"
    } else if model.starts_with("perplexity-") {
        "perplexity"
    } else {
        "anthropic"
    }
}

#[tauri::command]
pub async fn get_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let api_keys = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        SettingsApiKeys {
            anthropic: settings.anthropic_api_key.clone(),
            openai: settings.openai_api_key.clone(),
            google: settings.google_api_key.clone(),
            xai: settings.xai_api_key.clone(),
            perplexity: settings.perplexity_api_key.clone(),
        }
    };

    let mut models = Vec::new();
    let client = Client::new();

    if !api_keys.anthropic.is_empty() {
        match fetch_anthropic_models(&client, &api_keys.anthropic).await {
            Ok(m) => models.extend(m),
            Err(e) => println!("Failed to fetch Anthropic models: {}", e),
        }
    }

    if !api_keys.openai.is_empty() {
        match fetch_openai_models(&client, &api_keys.openai).await {
            Ok(m) => models.extend(m),
            Err(e) => println!("Failed to fetch OpenAI models: {}", e),
        }
    }

    if !api_keys.google.is_empty() {
        match fetch_google_models(&client, &api_keys.google).await {
            Ok(m) => models.extend(m),
            Err(e) => println!("Failed to fetch Google models: {}", e),
        }
    }

    if !api_keys.xai.is_empty() {
        match fetch_xai_models(&client, &api_keys.xai).await {
            Ok(m) => models.extend(m),
            Err(e) => println!("Failed to fetch xAI models: {}", e),
        }
    }

    if !api_keys.perplexity.is_empty() {
        match fetch_perplexity_models(&client, &api_keys.perplexity).await {
            Ok(m) => models.extend(m),
            Err(e) => println!("Failed to fetch Perplexity models: {}", e),
        }
    }

    if models.is_empty() {
        models.extend(vec![
            ModelInfo { id: "claude-sonnet-4-20250514".to_string(), name: "Claude Sonnet 4 (请先配置API Key)".to_string(), provider: "Anthropic".to_string() },
            ModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o (请先配置API Key)".to_string(), provider: "OpenAI".to_string() },
        ]);
    }

    Ok(models)
}

async fn fetch_anthropic_models(client: &Client, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let response = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let (Some(id), Some(display_name)) = (
                model.get("id").and_then(|v| v.as_str()),
                model.get("display_name").and_then(|v| v.as_str())
            ) {
                if !id.contains("claude-") && !id.contains("sonnet") && !id.contains("haiku") && !id.contains("opus") {
                    continue;
                }
                models.push(ModelInfo {
                    id: id.to_string(),
                    name: display_name.to_string(),
                    provider: "Anthropic".to_string(),
                });
            }
        }
    }

    if models.is_empty() {
        return Err("Failed to fetch Anthropic models - please check your API key".to_string());
    }

    Ok(models)
}

async fn fetch_openai_models(client: &Client, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let response = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
                let filter_models = ["gpt-4o", "gpt-4", "gpt-3.5", "o1", "o3", "o4"];
                if !filter_models.iter().any(|f| id.contains(f)) {
                    continue;
                }
                let name = model.get("human_name").and_then(|v| v.as_str()).unwrap_or(id);
                models.push(ModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    provider: "OpenAI".to_string(),
                });
            }
        }
    }

    models.sort_by(|a, b| {
        let priority = |id: &str| {
            if id.contains("4o") { 0 }
            else if id.contains("o1") { 1 }
            else if id.contains("o3") { 2 }
            else if id.contains("4") { 3 }
            else { 4 }
        };
        priority(&a.id).cmp(&priority(&b.id))
    });

    Ok(models)
}

async fn fetch_google_models(client: &Client, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let response = client
        .get(&format!("https://generativelanguage.googleapis.com/v1/models?key={}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(data) = json.get("models").and_then(|d| d.as_array()) {
        for model in data {
            if let Some(name) = model.get("name").and_then(|v| v.as_str()) {
                if !name.contains("gemini") {
                    continue;
                }
                let model_id = name.replace("models/", "");
                let display_name = model_id.replace("-", " ");
                models.push(ModelInfo {
                    id: model_id,
                    name: display_name,
                    provider: "Google".to_string(),
                });
            }
        }
    }

    if models.is_empty() {
        return Err("Failed to fetch Google models - please check your API key".to_string());
    }

    Ok(models)
}

async fn fetch_xai_models(client: &Client, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let response = client
        .get("https://api.x.ai/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
                let name = model.get("human_name").and_then(|v| v.as_str()).unwrap_or(id);
                models.push(ModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    provider: "xAI".to_string(),
                });
            }
        }
    }

    if models.is_empty() {
        return Err("Failed to fetch xAI models - please check your API key".to_string());
    }

    Ok(models)
}

async fn fetch_perplexity_models(client: &Client, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let response = client
        .get("https://api.perplexity.ai/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let (Some(id), Some(name)) = (
                model.get("id").and_then(|v| v.as_str()),
                model.get("name").and_then(|v| v.as_str())
            ) {
                models.push(ModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    provider: "Perplexity".to_string(),
                });
            }
        }
    }

    if models.is_empty() {
        return Err("Failed to fetch Perplexity models - please check your API key".to_string());
    }

    Ok(models)
}

#[tauri::command]
pub async fn chat(
    state: State<'_, AppState>,
    message: String,
    model: Option<String>,
    system_prompt: Option<String>,
) -> Result<String, String> {
    let (default_model, api_keys) = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        (settings.default_model.clone(), SettingsApiKeys {
            anthropic: settings.anthropic_api_key.clone(),
            openai: settings.openai_api_key.clone(),
            google: settings.google_api_key.clone(),
            xai: settings.xai_api_key.clone(),
            perplexity: settings.perplexity_api_key.clone(),
        })
    };
    
    let model = model.unwrap_or(default_model);
    let provider = get_model_provider(&model);

    let context = build_context(&state).map_err(|e| e.to_string())?;
    let full_message = if context.is_empty() {
        message.clone()
    } else {
        format!("{}\n\nUser: {}", context, message)
    };

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: message.clone(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let response = match provider {
        "anthropic" => chat_anthropic(&api_keys, &model, &full_message, system_prompt).await,
        "openai" => chat_openai(&api_keys, &model, &full_message, system_prompt).await,
        "google" => chat_google(&api_keys, &model, &full_message, system_prompt).await,
        "xai" => chat_xai(&api_keys, &model, &full_message, system_prompt).await,
        "perplexity" => chat_perplexity(&api_keys, &model, &full_message, system_prompt).await,
        _ => chat_anthropic(&api_keys, &model, &full_message, system_prompt).await,
    };

    let assistant_message = ChatMessage {
        role: "assistant".to_string(),
        content: response.clone().unwrap_or_else(|e| e.clone()),
        timestamp: chrono::Utc::now().timestamp(),
    };

    {
        let mut messages = state.messages.lock().map_err(|e| e.to_string())?;
        messages.push(user_message);
        if response.is_ok() {
            messages.push(assistant_message);
        }
    }

    response
}

#[derive(Clone)]
struct SettingsApiKeys {
    anthropic: String,
    openai: String,
    google: String,
    xai: String,
    perplexity: String,
}

fn build_context(state: &AppState) -> Result<String, String> {
    let messages = state.messages.lock().map_err(|e| e.to_string())?;
    let memories = state.memories.lock().map_err(|e| e.to_string())?;

    let mut context = String::new();

    if let Some(last_user_msg) = messages.iter().rev().find(|m| m.role == "user") {
        let query = extract_keywords(&last_user_msg.content);
        if !query.is_empty() {
            let relevant: Vec<_> = memories
                .iter()
                .filter(|m| {
                    m.title.to_lowercase().contains(&query)
                        || m.tags.iter().any(|t| t.to_lowercase().contains(&query))
                        || m.entities.iter().any(|e| e.to_lowercase().contains(&query))
                })
                .take(5)
                .collect();

            if !relevant.is_empty() {
                context.push_str("## Relevant Memories\n");
                for memory in &relevant {
                    context.push_str(&format!("### {}\n{}\n\n", memory.title, memory.content));
                }
            }
        }
    }

    if memories.is_empty() && !messages.is_empty() {
        context.push_str("## Recent Conversation\n");
        let recent: Vec<_> = messages.iter().rev().take(10).collect();
        for msg in recent.iter().rev() {
            context.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
    }

    Ok(context)
}

fn extract_keywords(text: &str) -> String {
    let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
        "may", "might", "must", "shall", "can", "need", "dare", "ought", "used", "to",
        "of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "through",
        "during", "before", "after", "above", "below", "between", "under", "again",
        "further", "then", "once", "here", "there", "when", "where", "why", "how",
        "all", "each", "few", "more", "most", "other", "some", "such", "no", "nor",
        "not", "only", "own", "same", "so", "than", "too", "very", "just", "and",
        "but", "if", "or", "because", "until", "while", "this", "that", "these",
        "those", "what", "which", "who", "whom", "i", "you", "he", "she", "it", "we",
        "they", "me", "him", "her", "us", "them", "my", "your", "his", "its", "our",
        "their", "mine", "yours", "hers", "ours", "theirs", "请", "帮我", "给我",
        "我想", "你能", "可以", "这个", "那个", "什么", "怎么", "如何", "为什么"];

    let words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .filter(|s| !stop_words.contains(s))
        .map(|s| s.to_string())
        .collect();

    words.join(" ")
}

async fn chat_anthropic(
    settings: &SettingsApiKeys,
    model: &str,
    message: &str,
    system_prompt: Option<String>,
) -> Result<String, String> {
    if settings.anthropic.is_empty() {
        return Err("Anthropic API key not configured".to_string());
    }

    let client = Client::new();
    
    let request = AnthropicRequest {
        model: model.to_string(),
        messages: vec![AnthropicMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }],
        max_tokens: 4096,
        system: system_prompt,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &settings.anthropic)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Anthropic API error ({}): {}", status, text));
    }

    let response: AnthropicResponse = response.json().await.map_err(|e| e.to_string())?;

    response
        .content
        .first()
        .and_then(|c| c.text.clone())
        .ok_or_else(|| "Empty response from Anthropic".to_string())
}

async fn chat_openai(
    settings: &SettingsApiKeys,
    model: &str,
    message: &str,
    system_prompt: Option<String>,
) -> Result<String, String> {
    if settings.openai.is_empty() {
        return Err("OpenAI API key not configured".to_string());
    }

    let client = Client::new();

    let mut messages = Vec::new();
    
    if let Some(system) = system_prompt {
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: system,
        });
    }
    
    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });

    let request = OpenAIRequest {
        model: model.to_string(),
        messages,
        max_tokens: Some(4096),
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", settings.openai))
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error ({}): {}", status, text));
    }

    let response: OpenAIResponse = response.json().await.map_err(|e| e.to_string())?;

    response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Empty response from OpenAI".to_string())
}

async fn chat_google(
    settings: &SettingsApiKeys,
    model: &str,
    message: &str,
    _system_prompt: Option<String>,
) -> Result<String, String> {
    if settings.google.is_empty() {
        return Err("Google API key not configured".to_string());
    }

    let client = Client::new();
    
    let _model_name = model.trim_start_matches("gemini-");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, settings.google
    );

    let request = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": message
            }]
        }],
        "generationConfig": {
            "maxOutputTokens": 4096,
            "temperature": 0.9
        }
    });

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Google API error ({}): {}", status, text));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    json.get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Empty response from Google".to_string())
}

async fn chat_xai(
    settings: &SettingsApiKeys,
    model: &str,
    message: &str,
    system_prompt: Option<String>,
) -> Result<String, String> {
    if settings.xai.is_empty() {
        return Err("xAI API key not configured".to_string());
    }

    let client = Client::new();

    let mut messages = Vec::new();
    
    if let Some(system) = system_prompt {
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: system,
        });
    }
    
    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });

    let request = OpenAIRequest {
        model: model.to_string(),
        messages,
        max_tokens: Some(4096),
    };

    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", settings.xai))
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("xAI API error ({}): {}", status, text));
    }

    let response: OpenAIResponse = response.json().await.map_err(|e| e.to_string())?;

    response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Empty response from xAI".to_string())
}

async fn chat_perplexity(
    settings: &SettingsApiKeys,
    model: &str,
    message: &str,
    _system_prompt: Option<String>,
) -> Result<String, String> {
    if settings.perplexity.is_empty() {
        return Err("Perplexity API key not configured".to_string());
    }

    let client = Client::new();

    let model_name = model.trim_start_matches("perplexity-");
    let url = "https://api.perplexity.ai/chat/completions";

    let request = OpenAIRequest {
        model: format!("llama-3.1-sonar-{}-128k-online", model_name),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }],
        max_tokens: Some(4096),
    };

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", settings.perplexity))
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error ({}): {}", status, text));
    }

    let response: OpenAIResponse = response.json().await.map_err(|e| e.to_string())?;

    response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Empty response from Perplexity".to_string())
}

use crate::{AppState, ChatMessage};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;

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

    if !memories.is_empty() {
        context.push_str("## Relevant Memories\n");
        for memory in memories.iter().take(5) {
            context.push_str(&format!("- {}\n", memory.title));
        }
        context.push('\n');
    }

    let recent_messages: Vec<_> = messages.iter().rev().take(10).collect();
    if !recent_messages.is_empty() {
        context.push_str("## Recent Conversation\n");
        for msg in recent_messages.iter().rev() {
            context.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
    }

    Ok(context)
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

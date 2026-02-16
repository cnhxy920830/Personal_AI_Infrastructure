use crate::{ChatMessage, MemoryItem, Settings};

pub struct HookSystem {
    message_count_threshold: usize,
    keywords: Vec<String>,
}

impl HookSystem {
    pub fn new() -> Self {
        Self {
            message_count_threshold: 5,
            keywords: vec![
                "记住".to_string(),
                "remember".to_string(),
                "memorize".to_string(),
                "重要".to_string(),
                "important".to_string(),
                "别忘了".to_string(),
                "don't forget".to_string(),
                "提醒我".to_string(),
                "remind me".to_string(),
                "学习".to_string(),
                "learn".to_string(),
                "项目".to_string(),
                "project".to_string(),
                "任务".to_string(),
                "task".to_string(),
                "会议".to_string(),
                "meeting".to_string(),
                "截止日期".to_string(),
                "deadline".to_string(),
                "人名".to_string(),
                "name".to_string(),
                "电话".to_string(),
                "phone".to_string(),
                "邮箱".to_string(),
                "email".to_string(),
                "地址".to_string(),
                "address".to_string(),
            ],
        }
    }

    pub fn check_and_extract_memory(&self, messages: &[ChatMessage], settings: &Settings) -> Option<MemoryItem> {
        let recent_messages: Vec<_> = messages.iter().rev().take(10).collect();
        
        for msg in &recent_messages {
            let content_lower = msg.content.to_lowercase();
            
            for keyword in &self.keywords {
                if content_lower.contains(&keyword.to_lowercase()) {
                    if let Some(memory) = self.analyze_and_extract(&msg.content, settings) {
                        return Some(memory);
                    }
                }
            }
        }

        if messages.len() >= self.message_count_threshold {
            if let Some(last_msg) = recent_messages.first() {
                if last_msg.role == "user" && last_msg.content.len() > 50 {
                    return self.analyze_contextual_memory(messages, settings);
                }
            }
        }

        None
    }

    fn analyze_and_extract(&self, content: &str, settings: &Settings) -> Option<MemoryItem> {
        let prompt = format!(
            r#"Analyze the following text and extract important information as a memory item.
Return a JSON object with these fields:
- title: A short descriptive title (max 50 characters)
- content: The main content to remember
- memory_type: One of WORK, LEARNING, RELATIONSHIP, or general
- tags: Array of relevant tags

Text to analyze:
{}

Respond with ONLY valid JSON, no other text."#,
            content
        );

        let response = self.call_ai_for_extraction(&prompt, settings)?;

        self.parse_memory_response(&response)
    }

    fn analyze_contextual_memory(&self, messages: &[ChatMessage], settings: &Settings) -> Option<MemoryItem> {
        let conversation: String = messages
            .iter()
            .rev()
            .take(5)
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"Analyze the following conversation and extract any important information that should be remembered.
Look for:
- User preferences or requirements
- Project or task details
- Important dates or deadlines
- Contact information
- Learning or knowledge gained
- Relationship details

Conversation:
{}

Respond with a JSON object with these fields:
- title: A short descriptive title (max 50 characters)
- content: The important information to remember
- memory_type: One of WORK, LEARNING, RELATIONSHIP, or general
- tags: Array of relevant tags

If nothing important found, respond with: {{"title": "", "content": "", "memory_type": "general", "tags": []}}"#,
            conversation
        );

        let response = self.call_ai_for_extraction(&prompt, settings)?;

        let memory = self.parse_memory_response(&response)?;
        
        if memory.title.is_empty() {
            return None;
        }

        Some(memory)
    }

    fn call_ai_for_extraction(&self, prompt: &str, settings: &Settings) -> Option<String> {
        let api_key = if !settings.anthropic_api_key.is_empty() {
            settings.anthropic_api_key.clone()
        } else if !settings.openai_api_key.is_empty() {
            settings.openai_api_key.clone()
        } else {
            return None;
        };

        let provider = if !settings.anthropic_api_key.is_empty() {
            "anthropic"
        } else {
            "openai"
        };

        let client = reqwest::blocking::Client::new();
        
        let body = if provider == "anthropic" {
            serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            })
        } else {
            serde_json::json!({
                "model": "gpt-4o-mini",
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            })
        };

        let url = if provider == "anthropic" {
            "https://api.anthropic.com/v1/messages"
        } else {
            "https://api.openai.com/v1/chat/completions"
        };

        let request = if provider == "anthropic" {
            client.post(url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
        } else {
            client.post(url)
                .header("authorization", format!("Bearer {}", api_key))
                .header("content-type", "application/json")
        };

        let response = request
            .json(&body)
            .send()
            .ok()?;

        let json: serde_json::Value = response.json().ok()?;
        
        let content = if provider == "anthropic" {
            json["content"][0]["text"].as_str()?.to_string()
        } else {
            json["choices"][0]["message"]["content"].as_str()?.to_string()
        };

        Some(content)
    }

    fn parse_memory_response(&self, response: &str) -> Option<MemoryItem> {
        let json_str = response.trim();
        let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

        let title = json["title"].as_str()?.to_string();
        let content = json["content"].as_str()?.to_string();
        let memory_type = json["memory_type"].as_str()?.to_string();
        
        let tags: Vec<String> = json["tags"]
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        Some(MemoryItem {
            id: format!("memory-{}", chrono::Utc::now().timestamp_millis()),
            title,
            content,
            memory_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            tags,
            entities: Vec::new(),
            confidence: 0.8,
        })
    }

    pub fn should_auto_extract(&self, message_count: usize) -> bool {
        message_count > 0 && message_count % self.message_count_threshold == 0
    }
}

impl Default for HookSystem {
    fn default() -> Self {
        Self::new()
    }
}

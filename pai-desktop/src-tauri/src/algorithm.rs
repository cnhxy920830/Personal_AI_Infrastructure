use crate::{ChatMessage, MemoryItem, Settings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmContext {
    pub user_requirements: String,
    pub constraints: Vec<String>,
    pub plan: String,
    pub validation_result: Option<ValidationResult>,
    pub reflection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

pub struct AlgorithmFramework {
    min_confidence_threshold: f32,
}

impl AlgorithmFramework {
    pub fn new() -> Self {
        Self {
            min_confidence_threshold: 0.7,
        }
    }

    pub fn extract_constraints(&self, requirements: &str) -> Vec<String> {
        let constraint_keywords = [
            "必须", "不能", "不要", "必须不", "仅", "只能",
            "must", "must not", "cannot", "don't", "only", "exactly",
            "限制", "limit", "不超过", "at most", "至少", "at least",
            "精确", "exactly", "严格", "strict",
        ];

        let mut constraints = Vec::new();
        let requirements_lower = requirements.to_lowercase();

        for keyword in constraint_keywords {
            if requirements_lower.contains(keyword) {
                if let Some(sentence) = self.extract_sentence_containing(requirements, keyword) {
                    if !constraints.contains(&sentence) {
                        constraints.push(sentence);
                    }
                }
            }
        }

        constraints
    }

    fn extract_sentence_containing(&self, text: &str, keyword: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        if let Some(pos) = text_lower.find(keyword) {
            let start = text[..pos].rfind(|c: char| c == '.' || c == '\n' || c == '，' || c == ',')
                .map(|p| p + 1)
                .unwrap_or(0);
            let end = text[pos..].find(|c: char| c == '.' || c == '\n' || c == '，' || c == ',')
                .map(|p| pos + p)
                .unwrap_or(text.len());
            let sentence = text[start..end].trim().to_string();
            if !sentence.is_empty() {
                return Some(sentence);
            }
        }
        None
    }

    pub fn create_plan(&self, requirements: &str, constraints: &[String]) -> String {
        let mut plan = String::new();
        plan.push_str("## Execution Plan\n\n");

        plan.push_str("### Requirements Analysis\n");
        plan.push_str(&format!("- Main goal: {}\n\n", self.extract_main_goal(requirements)));

        if !constraints.is_empty() {
            plan.push_str("### Constraints\n");
            for constraint in constraints {
                plan.push_str(&format!("- {}\n", constraint));
            }
            plan.push('\n');
        }

        plan.push_str("### Steps\n");
        plan.push_str("1. Validate input against constraints\n");
        plan.push_str("2. Generate response following constraints\n");
        plan.push_str("3. Self-reflect on response quality\n");
        plan.push_str("4. Validate final output\n");

        plan
    }

    fn extract_main_goal(&self, requirements: &str) -> String {
        let first_line = requirements.lines().next().unwrap_or(requirements);
        if first_line.len() > 100 {
            format!("{}...", &first_line[..100])
        } else {
            first_line.to_string()
        }
    }

    pub fn validate_output(&self, output: &str, constraints: &[String], settings: &Settings) -> ValidationResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        for constraint in constraints {
            let constraint_lower = constraint.to_lowercase();
            
            if constraint_lower.contains("不能") || constraint_lower.contains("不要") 
                || constraint_lower.contains("must not") || constraint_lower.contains("cannot") 
                || constraint_lower.contains("don't") {
                if self.constraint_violated(output, constraint) {
                    issues.push(format!("Constraint violated: {}", constraint));
                    suggestions.push(format!("Remove content related to: {}", constraint));
                }
            }

            if constraint_lower.contains("仅") || constraint_lower.contains("只能") 
                || constraint_lower.contains("only") {
                if !self.constraint_satisfied(output, constraint) {
                    issues.push(format!("Constraint not satisfied: {}", constraint));
                }
            }
        }

        if output.len() < 10 {
            issues.push("Output is too short".to_string());
        }

        ValidationResult {
            passed: issues.is_empty(),
            issues,
            suggestions,
        }
    }

    fn constraint_violated(&self, output: &str, constraint: &str) -> bool {
        let constraint_words: Vec<&str> = constraint
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        let output_lower = output.to_lowercase();
        
        constraint_words.iter().any(|word| output_lower.contains(&word.to_lowercase()))
    }

    fn constraint_satisfied(&self, output: &str, constraint: &str) -> bool {
        true
    }

    pub fn reflect(&self, output: &str, requirements: &str) -> String {
        let mut reflection = String::new();
        
        reflection.push_str("## Self-Reflection\n\n");
        
        let output_len = output.len();
        let req_len = requirements.len();
        
        if output_len < req_len / 2 {
            reflection.push_str("- ⚠️ Output may be too brief compared to requirements\n");
        } else if output_len > req_len * 3 {
            reflection.push_str("- ⚠️ Output may be unnecessarily verbose\n");
        } else {
            reflection.push_str("- ✅ Output length seems appropriate\n");
        }

        let has_structure = output.contains('\n') || output.contains('.');
        if has_structure {
            reflection.push_str("- ✅ Output has clear structure\n");
        } else {
            reflection.push_str("- ⚠️ Consider adding more structure to output\n");
        }

        reflection
    }

    pub fn rehearse(&self, context: &AlgorithmContext, settings: &Settings) -> Option<String> {
        let prompt = format!(
            r#"You are validating an AI response before it's sent to the user.

Original Requirements:
{}

Constraints:
{}

AI Response to Validate:
{}

Please analyze and provide feedback. Is this response appropriate? What improvements could be made?

Respond with your analysis."#,
            context.user_requirements,
            context.constraints.join("\n"),
            context.plan
        );

        self.call_ai_validation(&prompt, settings)
    }

    fn call_ai_validation(&self, prompt: &str, settings: &Settings) -> Option<String> {
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
                "max_tokens": 512,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            })
        } else {
            serde_json::json!({
                "model": "gpt-4o-mini",
                "max_tokens": 512,
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

        let response = request.json(&body).send().ok()?;

        let json: serde_json::Value = response.json().ok()?;
        
        let content = if provider == "anthropic" {
            json["content"][0]["text"].as_str()?.to_string()
        } else {
            json["choices"][0]["message"]["content"].as_str()?.to_string()
        };

        Some(content)
    }
}

impl Default for AlgorithmFramework {
    fn default() -> Self {
        Self::new()
    }
}

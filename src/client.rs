use std::env;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dotenv;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Model,
}

#[derive(Serialize)]
pub struct MessagePart {
    pub text: String,
}

#[derive(Serialize)]
pub struct Message {
    role: MessageRole,
    parts: Vec<MessagePart>,
}

impl Message {
    pub fn new(role: MessageRole, text: &str) -> Self {
        Self {
            role,
            parts: vec![MessagePart {
                text: text.to_string(),
            }],
        }
    }
}

#[derive(Serialize)]
pub struct ChatRequest {
    contents: Vec<Message>,
}

impl ChatRequest {
    pub fn new(messages: Vec<Message>) -> Self {
        Self { contents: messages }
    }
}

#[derive(Deserialize)]
pub struct IdentifierCandidate {
    pub name: String,
    pub desc: String,
}

pub struct Client {
    url: String,
    http_client: reqwest::Client,
    api_key: String,
}

impl Client {
    pub fn new() -> Result<Self, anyhow::Error> {
        dotenv::load();

        let http_client = reqwest::Client::new();
        let url =
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent"
                .to_string();
        let api_key = env::var("GEMINI_API_KEY")?;

        Ok(Self {
            url,
            http_client,
            api_key,
        })
    }

    pub async fn get_candidates(
        &self,
        text: &str,
    ) -> Result<Vec<IdentifierCandidate>, anyhow::Error> {
        let request = self.build_request(text);

        let response_text = self
            .http_client
            .post(&self.url)
            .query(&[("key", &self.api_key)])
            .body(serde_json::to_string(&request)?)
            .send()
            .await?
            .text()
            .await?;

        let json_res: Value = serde_json::from_str(&response_text)?;

        if let Some(error) = json_res.get("error") {
            return Err(anyhow!(error.to_string()));
        }

        let raw_answer = &json_res["candidates"][0]["content"]["parts"][0]["text"];
        if raw_answer.is_null() {
            return Err(anyhow!("Unexpected error."));
        }

        let raw_answer = raw_answer
            .to_string()
            .trim_matches('"')
            .replace("\\n", "")
            .replace("\\", "");

        let candidates: Vec<IdentifierCandidate> = serde_json::from_str(&raw_answer)?;
        Ok(candidates)
    }

    fn build_request(&self, text: &str) -> ChatRequest {
        ChatRequest::new(vec![
            Message::new(
                MessageRole::User,
                r#"
                    これから入力する文章から、候補となるTypeScriptの識別子をその識別子の簡単な特徴とともに5個列挙してください。
                    出力はJSON形式で行ってください。
                "#,
            ),
            Message::new(
                MessageRole::Model,
                r#"分かりました。文章を入力してください。"#,
            ),
            Message::new(MessageRole::User, "タスクを作成する関数"),
            Message::new(
                MessageRole::Model,
                r#"[
                    {"name": "createTask", "desc": "動詞+名詞の組み合わせで、関数の目的を明確に表す一般的な命名規則"},
                    {"name": "addTask", "desc": "`createTask`と同様に関数の目的を表すが、より簡潔"},
                    {"name": "newTask", "desc": "新しいタスクを作成するという意味を強調"},
                    {"name": "generateTask", "desc": "タスクを自動的に生成するような場合に適している"},
                    {"name": "buildTask", "desc": "タスクを構築するイメージ"}
                ]"#,
            ),
            Message::new(MessageRole::User, text),
        ])
    }
}

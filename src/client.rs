use std::env;

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::dotenv;

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
        let body = json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": r#"
                        これから入力する文章から、候補となるTypeScriptの識別子をその識別子の簡単な特徴とともに5個列挙してください。
                        出力はJSON形式で行ってください。
                    "#}]
                },
                {
                    "role": "model",
                    "parts": [{"text": "分かりました。文章を入力してください。"}]
                },
                {
                    "role": "user",
                    "parts": [{"text": "タスクを作成する関数"}]
                },
                {
                    "role": "model",
                    "parts": [{"text": r#"[
                        {"name": "createTask", "desc": "動詞+名詞の組み合わせで、関数の目的を明確に表す一般的な命名規則"},
                        {"name": "addTask", "desc": "`createTask`と同様に関数の目的を表すが、より簡潔"},
                        {"name": "newTask", "desc": "新しいタスクを作成するという意味を強調"},
                        {"name": "generateTask", "desc": "タスクを自動的に生成するような場合に適している"},
                        {"name": "buildTask", "desc": "タスクを構築するイメージ"}
                    ]"#}]
                },
                {
                    "role": "user",
                    "parts": [{"text": text}]
                },
            ]
        });

        let response = self
            .http_client
            .post(&self.url)
            .query(&[("key", &self.api_key)])
            .body(body.to_string())
            .send()
            .await?;

        let text = response.text().await?;
        let json: Value = serde_json::from_str(&text)?;

        if let Some(error) = json.get("error") {
            return Err(anyhow!(error.to_string()));
        }

        let raw_answer = &json["candidates"][0]["content"]["parts"][0]["text"];
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
}

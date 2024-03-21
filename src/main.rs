use std::{env, error::Error};

use dotenvy::dotenv;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env file not found");

    let args = env::args().collect::<Vec<String>>();
    let input = args.get(1).unwrap();

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
                "parts": [{"text": r#"{
                    "candidates": [
                        {"name": "createTask", "desc": "動詞+名詞の組み合わせで、関数の目的を明確に表す一般的な命名規則"},
                        {"name": "addTask", "desc": "`createTask`と同様に関数の目的を表すが、より簡潔"},
                        {"name": "newTask", "desc": "新しいタスクを作成するという意味を強調"},
                        {"name": "generateTask", "desc": "タスクを自動的に生成するような場合に適している"},
                        {"name": "buildTask", "desc": "タスクを構築するイメージ"}
                    ]
                }"#}]
            },
            {
                "role": "user",
                "parts": [{"text": input}]
            },
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent")
        .query(&[("key", env::var("GEMINI_API_KEY").unwrap())])
        .body(body.to_string())
        .send()
        .await?;
    let json: Value = serde_json::from_str(&response.text().await?).unwrap();

    if let Some(error) = json.get("error") {
        println!("{}", error.to_string());
        return Ok(());
    }

    let raw_answer = &json["candidates"][0]["content"]["parts"][0]["text"];
    if raw_answer.is_null() {
        println!("Unexpected error.");
        return Ok(());
    }

    let raw_answer = raw_answer
        .to_string()
        .trim_matches('"')
        .replace("\\n", "")
        .replace("\\", "");

    let answer: Value = serde_json::from_str(&raw_answer).unwrap();

    println!("{}", serde_json::to_string_pretty(&answer).unwrap());
    Ok(())
}

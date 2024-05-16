use dotenv::dotenv;
use reqwest::{
    self,
    header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
};
use tokio;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let token = std::env::var("TOKEN").unwrap_or_else(|err| {
        println!("No token found in .env file: {}\n Getting a new one", err);
        let registration_future = async {
            match register_new_agent().await {
                Ok(json_body) => {
                    let token = json_body["token"].as_str().unwrap_or("").to_string();
                    let file = File::create("agent.json").unwrap();
                    let mut writer = BufWriter::new(file);
                    serde_json::to_writer_pretty(&mut writer, &json_body).unwrap();
                    writer.flush().unwrap();
                    let mut env_file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(".env")
                    .unwrap();
        
                writeln!(env_file, "TOKEN={}", token).unwrap();
                    token
                }
                Err(e) => {
                    eprintln!("Error registering agent: {}", e);
                    String::new() // Return a default value in case of error
                }
            }
        };
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(registration_future)
    });
    let agent_data = get_agent_data(&token).await.unwrap();
    print!("{}", agent_data);
    Ok(())
}

async fn register_new_agent() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let data = r#"{
    "symbol": "SFP-1",
    "faction": "COSMIC"
    }"#;

    let json: serde_json::Value = serde_json::from_str(&data)?;

    let request = client
        .request(
            reqwest::Method::POST,
            "https://api.spacetraders.io/v2/register",
        )
        .headers(headers)
        .json(&json);

    let response = request.send().await?;
    let body = response.json::<serde_json::Value>().await?;

    Ok(body)
}

async fn get_agent_data(token: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::GET,
            "https://api.spacetraders.io/v2/my/agent",
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<serde_json::Value>().await?;


    Ok(body)
}

use dotenv::dotenv;
use reqwest::{
    self,
    header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write}, num,
};
use tokio;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Contract {
    accepted: bool,
    #[serde(rename = "deadlineToAccept")]
    deadline_to_accept: String,
    #[serde(rename = "expiration")]
    expiration_date: String,
    #[serde(rename = "factionSymbol")]
    faction_symbol: String,
    fulfilled: bool,
    id: String,
    terms: Terms,
    #[serde(rename = "type")]
    tpye: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Terms {
    deadline: String,
    #[serde(rename = "deliver")]
    deliveries: Vec<Delivery>,
    #[serde(rename = "payment")]
    payment: Payment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "onAccepted")]
    payment_on_accepted: u64,
    #[serde(rename = "onFulfilled")]
    payment_on_fulfilled: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Delivery {
    #[serde(rename = "destinationSymbol")]
    destination_symbol: String,
    #[serde(rename = "tradeSymbol")]
    trade_symbol: String,
    #[serde(rename = "unitsFulfilled")]
    units_fulfilled: u64,
    #[serde(rename = "unitsRequired")]
    units_required: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    limit: u32,
    page: u32,
    total: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    data: Vec<Contract>,
    meta: Meta,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let token = check_for_token().await;

    let agent_data = get_agent_data(&token).await.unwrap();
    println!("{}", agent_data);
    let agent_contracts = get_contracts(&token).await.unwrap();
    // for contract in agent_contracts {
    //     accept_contract(&token, &contract.id).await.unwrap();
    // }
    Ok(())
}

async fn check_for_token() -> String {
    return std::env::var("TOKEN").unwrap_or_else(|err| {
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


async fn get_contracts(token: &str) -> Result<Vec<Contract>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::GET,
            "https://api.spacetraders.io/v2/my/contracts",
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<Response>().await?;
    let contracts = body.data;
    Ok(contracts)
}

async fn accept_contract(token: &str, contract_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/contracts/{}/accept", contract_id),
        )
        .headers(headers);

    let _response = request.send().await?;

    Ok(())
}

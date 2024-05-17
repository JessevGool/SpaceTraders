use base64;
use dotenv::dotenv;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{
    self,
    header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Value;
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
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

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Contract {{ \
            accepted: {}, \
            deadline_to_accept: {}, \
            expiration_date: {}, \
            faction_symbol: {}, \
            fulfilled: {}, \
            id: {}, \
            terms: {}, \
            type: {} }}",
            self.accepted,
            self.deadline_to_accept,
            self.expiration_date,
            self.faction_symbol,
            self.fulfilled,
            self.id,
            self.terms,
            self.tpye
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Terms {
    deadline: String,
    #[serde(rename = "deliver")]
    deliveries: Vec<Delivery>,
    #[serde(rename = "payment")]
    payment: Payment,
}

impl fmt::Display for Terms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Terms {{ \
            deadline: {}, \
            deliveries: [], \
            payment: {} }}",
            self.deadline, self.payment
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "onAccepted")]
    payment_on_accepted: u64,
    #[serde(rename = "onFulfilled")]
    payment_on_fulfilled: u64,
}

impl fmt::Display for Payment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Payment {{ \
            payment_on_accepted: {}, \
            payment_on_fulfilled: {} }}",
            self.payment_on_accepted, self.payment_on_fulfilled
        )
    }
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
pub struct AgentData {
    #[serde(rename = "accountId")]
    account_id: String,
    credits: u64,
    headquarters: String,
    #[serde(rename = "shipCount")]
    ship_count: u64,
    #[serde(rename = "startingFaction")]
    starting_faction: String,
    symbol: String,
}

impl fmt::Display for Delivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Delivery {{ \
            destination_symbol: {}, \
            trade_symbol: {}, \
            units_fulfilled: {}, \
            units_required: {} }}",
            self.destination_symbol, self.trade_symbol, self.units_fulfilled, self.units_required
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    limit: u32,
    page: u32,
    total: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContractResponse {
    data: Vec<Contract>,
    meta: Meta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentDataResponse {
    data: AgentData,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ViewAvailableShipsResponse {
   data: AvailableShips,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableShips {
    symbol: String,
    #[serde(rename = "shipTypes")]
    ship_types: Vec<ShipType>,
    #[serde(rename = "modificationsFee")]
    modifications_fee: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShipType {
    #[serde(rename = "type")]
   type_: String,
}



#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let token = check_for_token().await;
    let agent_data = get_agent_data(&token).await.unwrap();
    let ship_yards = find_shipyards(&token, "X1-TM62").await.unwrap();
    
    for ship_yard in ship_yards {
        let ships = find_ships_at_shipyard(&token, &ship_yard.system_symbol, &ship_yard.symbol).await.unwrap();
        println!("{:?}", ships);
    }

    

    let agent_contracts = get_contracts(&token).await.unwrap();
    for contract in agent_contracts {
        println!("{:?}", contract.to_string());
        // if !contract.accepted {
        //     let response = accept_contract(&token, &contract.id).await.unwrap();
        //     println!("{:?}", response);
        // }
    }
    Ok(())
}

async fn find_ships_at_shipyard(token:  &str, system_symbol:  &str, shipyard_symbol: &str) -> Result<AvailableShips, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());

    let request = client
        .request(
            reqwest::Method::GET,
            &format!(
                "https://api.spacetraders.io/v2/systems/{}/waypoints/{}/shipyard",
                system_symbol, shipyard_symbol
            ),
        )
        .headers(headers);

    let response = request.send().await.unwrap();
    //The ViewAvailableShipsResponse struct is inside of the data field of the response
    let body = response.json::<ViewAvailableShipsResponse>().await.unwrap();
    Ok(body.data)

    
}

async fn check_for_token() -> String {
    let has_token = std::env::var("TOKEN").is_ok();
    let mut token = String::new();
    if !has_token {
        let response = register_new_agent().await.unwrap();
        token = response["data"]["token"].as_str().unwrap_or("").to_string();
        let file = File::create("agent.json").unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &response).unwrap();
        writer.flush().unwrap();
        let mut env_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(".env")
            .unwrap();

        writeln!(env_file, "TOKEN={}", token).unwrap();
    } else {
        token = std::env::var("TOKEN").unwrap();
    }
    token
}

fn generate_random_symbol() -> String {
    let random_bytes: Vec<u8> = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(|c| c as u8)
        .take(6)
        .collect(); // Collect into a Vec<u8>

    format!("SFP-{}", base64::encode(&random_bytes))
}

async fn register_new_agent() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let symbol = generate_random_symbol();
    let data = format!(
        r#"{{
    "symbol": "{}",
    "faction": "COSMIC"
    }}"#,
        symbol
    );

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

async fn get_agent_data(token: &str) -> Result<AgentData, Box<dyn std::error::Error>> {
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
    let body = response.json::<AgentDataResponse>().await?;
    let agent_data = body.data;
   
    Ok(agent_data)
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
    let body = response.json::<ContractResponse>().await?;
    let contracts = body.data;
    Ok(contracts)
}

async fn accept_contract(
    token: &str,
    contract_id: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::POST,
            &format!(
                "https://api.spacetraders.io/v2/my/contracts/{}/accept",
                contract_id
            ),
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<serde_json::Value>().await?;
    Ok(body)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chart {
    #[serde(rename = "submittedBy")]
    submitted_by: String,
    #[serde(rename = "submittedOn")]
    submitted_on: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Faction {
    symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trait {
    description: String,
    name: String,
    symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct System {
    chart: Chart,
    faction: Faction,
    #[serde(rename = "isUnderConstruction")]
    is_under_construction: bool,
    modifiers: Vec<String>, // Assuming modifiers are strings for simplicity
    orbitals: Vec<String>, // Assuming orbitals are strings for simplicity
    orbits: String,
    symbol: String,
    #[serde(rename = "systemSymbol")]
    system_symbol: String,
    traits: Vec<Trait>,
    #[serde(rename = "type")]
    type_: String,
    x: i32,
    y: i32,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct FindShipyardResponse {
    data: Vec<System>,
    meta: Meta,
}


async fn find_shipyards(token: &str, system: &str) -> Result<Vec<System>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::GET,
            &format!(
                "https://api.spacetraders.io/v2/systems/{}/waypoints?traits=SHIPYARD",
                system
            ),
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<FindShipyardResponse>().await?;
        println!("{:?}", body.data.len());
    let systems = body.data;

    Ok(systems)
}

use base64;
use dotenv::dotenv;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{
    self,
    header::{HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
};

use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufWriter, Write}, time::Duration,
};
use tokio::{self, time::sleep};

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
    contract_type: String,
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
            self.contract_type
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
            deliveries: {:?}, \
            payment: {} }}",
            self.deadline,self.deliveries, self.payment
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
    #[serde(default)]
    symbol: Option<String>,
    #[serde(rename = "shipTypes", default)]
    ship_types: Option<Vec<ShipType>>,
    #[serde(default)]
    transactions: Option<Vec<Transaction>>,
    #[serde(default)]
    ships: Option<Vec<Ship>>,
    #[serde(rename = "modificationsFee", default)]
    modifications_fee: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "shipSymbol", default)]
    ship_symbol: Option<String>,
    #[serde(rename = "shipType", default)]
    ship_type: Option<String>,
    #[serde(rename = "waypointSymbol", default)]
    waypoint_symbol: Option<String>,
    #[serde(default)]
    price: Option<u64>,
    #[serde(default)]
    timestamp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ship {
    #[serde(rename = "type", default)]
    ship_type: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    supply: Option<String>,
    #[serde(default)]
    activity: Option<String>,
    #[serde(rename = "purchasePrice", default)]
    purchase_price: Option<u64>,
    frame: Frame,
    reactor: Reactor,
    engine: Engine,
    #[serde(default)]
    modules: Option<Vec<Module>>,
    #[serde(default)]
    mounts: Option<Vec<Mount>>,
    crew: Crew,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "moduleSlots", default)]
    module_slots: Option<u32>,
    #[serde(rename = "mountingPoints", default)]
    mounting_points: Option<u32>,
    #[serde(rename = "fuelCapacity", default)]
    fuel_capacity: Option<u32>,
    #[serde(default)]
    quality: Option<u32>,
    requirements: Requirements,
    #[serde(default)]
    condition: Option<u32>,
    #[serde(default)]
    integrity: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reactor {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "powerOutput", default)]
    power_output: Option<u32>,
    #[serde(default)]
    quality: Option<u32>,
    requirements: Requirements,
    #[serde(default)]
    condition: Option<u32>,
    #[serde(default)]
    integrity: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Engine {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    speed: Option<u32>,
    #[serde(default)]
    quality: Option<u32>,
    requirements: Requirements,
    #[serde(default)]
    condition: Option<u32>,
    #[serde(default)]
    integrity: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mount {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    strength: Option<u32>,
    #[serde(default)]
    deposits: Option<Vec<String>>,
    requirements: Requirements,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Crew {
    #[serde(default)]
    required: Option<u32>,
    #[serde(default)]
    capacity: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Requirements {
    #[serde(default)]
    power: Option<u32>,
    #[serde(default)]
    crew: Option<i32>,
    #[serde(default)]
    slots: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    capacity: Option<u32>,
    requirements: Requirements,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShipType {
    #[serde(rename = "type", default)]
    ship_type: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let token = check_for_token().await;
    let agent_data = get_agent_data(&token).await.unwrap();
    let ship_yards = find_shipyards(&token, "X1-VM68").await.unwrap();

    // for ship_yard in ship_yards {
    //     let ships = find_ships_at_shipyard(&token, &ship_yard.system_symbol, &ship_yard.symbol)
    //         .await
    //         .unwrap();
    //     if ships.ships.is_some() {
    //         for ship in ships.ships.unwrap() {

    //             if ship.ship_type.is_some() {
    //                 if ship.ship_type.unwrap() == "SHIP_MINING_DRONE" {
    //                     // let response = buy_ship(&token, &ship_yard.symbol, "SHIP_MINING_DRONE")
    //                     //     .await
    //                     //     .unwrap();
    //                     // println!("{:?}", response);
    //                 }
    //             }
    //         }
    //     }
    // }

    let engineered_asteroid_response = waypoint_by_type(&token, "X1-VM68", "ENGINEERED_ASTEROID")
        .await
        .unwrap();
    //  println!("{:?}", engineered_asteroid_response);
    let waypoint_symbol = engineered_asteroid_response["data"][0]["symbol"]
        .as_str()
        .unwrap();
    //Send ship to orbit
    // let orbit_response = send_ship_to_orbit(&token, "SFP-T1LRRNJW-4").await.unwrap();
    // println!("{:?}", orbit_response);

    let ship_id = "SFP-T1LRRNJW-4";

    // let navigate_response = navigate_to_waypoint(&token, &ship_id, &waypoint_symbol).await.unwrap();
    // println!("{:?}", navigate_response);

    // let dock_response = dock_ship(&token, &ship_id).await.unwrap();
    // println!("{:?}", dock_response);

    // let refuel_response = refuel_ship(&token, &ship_id).await.unwrap();
    // println!("{:?}", refuel_response);

    // let orbit_response = send_ship_to_orbit(&token, &ship_id).await.unwrap();
    // println!("{:?}", orbit_response);

   

    let agent_contracts = get_contracts(&token).await.unwrap();
    for contract in agent_contracts {
        println!("{:?}", contract.to_string());
        if !contract.accepted {
          
            let response = accept_contract(&token, &contract.id).await.unwrap();
            println!("{:?}", response);
        }
    }
    process_extraction(&token, ship_id).await;
    Ok(())
}


async fn process_extraction(token: &str, ship_id: &str) {
    loop {
        let extract_response = extract_ores(token, ship_id).await.unwrap();
        
        if!extract_response["error"].is_null() {
            if extract_response["error"]["code"] == 4000 {
                let cooldown = extract_response["error"]["data"]["cooldown"]["remainingSeconds"]
                   .as_u64()
                   .unwrap_or(0); // Use 0 if the value cannot be converted
                
                if cooldown > 0 {
                    println!("Cooldown: {}", cooldown);
                    
                    // Sleep for the duration of the cooldown
                    sleep(Duration::from_secs(cooldown)).await;
                }
            }
        } else {
          
            let ship_cargo = get_ship_cargo(token, ship_id).await.unwrap();
            let capacity = ship_cargo.capacity;
            let mut total_units = 0;
            for cargo in &ship_cargo.inventory {
                total_units += cargo.units;
            }
            if total_units == capacity {
                let dock_response = dock_ship(token, ship_id).await.unwrap();
                for cargo in ship_cargo.inventory {
                    if cargo.symbol != "COPPER_ORE" {
                    let sell_response = sell_goods(token, ship_id, &cargo.symbol, &cargo.units).await.unwrap();
                    println!("{:?}", sell_response);
                }
            }
             
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCargoResponse {
    data: Cargo,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Cargo {
    capacity: u32,
    inventory: Vec<CargoObject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CargoObject {
    description: String,
    name: String,
    symbol: String,
    units: u32,
}
async fn get_ship_cargo(
    token: &str,
    ship_id: &str,
) -> Result<Cargo, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());

    let request = client
        .request(
            reqwest::Method::GET,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/cargo", ship_id),
        )
        .headers(headers);

    let response = request.send().await.unwrap();
    let body = response.json::<GetCargoResponse>().await.unwrap();
    Ok(body.data)
}

async fn sell_goods(token: &str, ship_id: &str, goods: &str, units: &u32) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    let data = format!(
        r#"{{
    "symbol": "{}",
    "units": {}
    }}"#,
    goods, units
    );

    let json: serde_json::Value = serde_json::from_str(&data).unwrap();


    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/sell", ship_id),
        )
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
}

async fn navigate_to_waypoint(
    token: &str,
    ship_id: &str,
    waypoint_symbol: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());

    let data = format!(
        r#"{{
    "waypointSymbol": "{}"
    }}"#,
        waypoint_symbol
    );

    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    let request = client
        .request(
            reqwest::Method::POST,
            &format!(
                "https://api.spacetraders.io/v2/my/ships/{}/navigate",
                ship_id
            ),
        )
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
}

async fn dock_ship(
    token: &str,
    ship_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    //Add empty content length header
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/dock", ship_id),
        )
        .headers(headers);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
}


async fn extract_ores(
    token: &str,
    ship_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    //Add empty content length header
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/extract", ship_id),
        )
        .headers(headers);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
}

async fn refuel_ship(
    token: &str,
    ship_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    //Add empty content length header
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/refuel", ship_id),
        )
        .headers(headers);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
}

async fn find_ships_at_shipyard(
    token: &str,
    system_symbol: &str,
    shipyard_symbol: &str,
) -> Result<AvailableShips, Box<dyn std::error::Error>> {
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
    let body = response.json::<ViewAvailableShipsResponse>().await.unwrap();
    Ok(body.data)
}

async fn buy_ship(
    token: &str,
    waypoint_symbol: &str,
    ship_type: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());

    let data = format!(
        r#"{{
    "shipType": "{}",
    "waypointSymbol": "{}"
    }}"#,
        ship_type, waypoint_symbol
    );

    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    let request = client
        .request(
            reqwest::Method::POST,
            "https://api.spacetraders.io/v2/my/ships",
        )
        .headers(headers)
        .json(&json);

    let response = request.send().await.unwrap();
    let body = response.json::<serde_json::Value>().await.unwrap();
    Ok(body)
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
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    //Add empty content length header
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

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
    modifiers: Vec<String>,
    orbitals: Vec<String>,
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

async fn find_shipyards(
    token: &str,
    system: &str,
) -> Result<Vec<System>, Box<dyn std::error::Error>> {
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
    let systems = body.data;

    Ok(systems)
}

async fn waypoint_by_type(
    token: &str,
    system: &str,
    waypoint_type: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    let request = client
        .request(
            reqwest::Method::GET,
            &format!(
                "https://api.spacetraders.io/v2/systems/{}/waypoints?type={}",
                system, waypoint_type
            ),
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<serde_json::Value>().await?;

    Ok(body)
}

async fn send_ship_to_orbit(
    token: &str,
    ship_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    //Add empty content length header
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

    let request = client
        .request(
            reqwest::Method::POST,
            &format!("https://api.spacetraders.io/v2/my/ships/{}/orbit", ship_id),
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<serde_json::Value>().await?;
    Ok(body)
}

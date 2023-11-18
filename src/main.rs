use std::error::Error;

use clap::Parser;
use log::{debug, info};
use reqwest::header::HeaderValue;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Parser)]
struct Args {
    router_host: String,
    router_port: u16,
    password: String,
}

#[derive(Deserialize, Debug)]
struct Ld {
    #[serde(rename = "LD")]
    token: String,
}

#[derive(Deserialize, Debug)]
enum LogonResult {
    #[serde(rename = "0")]
    Success,
    #[serde(rename = "3")]
    Error,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
struct Logon {
    result: LogonResult,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    let get_url = format!(
        "http://{}:{}/goform/goform_get_cmd_process",
        args.router_host, args.router_port
    );
    let post_url = format!(
        "http://{}:{}/goform/goform_set_cmd_process",
        args.router_host, args.router_port
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.append(
        "Referer",
        HeaderValue::from_str(&format!("http://{}/", args.router_host))?,
    );
    headers.append("Host", HeaderValue::from_str(&args.router_host)?);

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .cookie_store(true)
        .build()?;

    // Start by getting a token from the server.
    let req = client
        .get(&get_url)
        .query(&[("isTest", "false"), ("cmd", "LD")])
        .build()?;

    let Ld { token } = client.execute(req).await?.json::<Ld>().await?;

    debug!("got token {token}");

    // Password = hex_upper(sha256(hex_upper(sha256(password)) + token))
    let mut hasher = Sha256::new();
    hasher.update(args.password.as_bytes());

    let mut password = [0u8; 128];

    let (left, right) = password.split_at_mut(64);
    base16ct::upper::encode(&hasher.finalize(), left).unwrap();

    right.copy_from_slice(token.as_bytes());

    let mut hasher = Sha256::new();
    hasher.update(password);

    let password = base16ct::upper::encode_str(&hasher.finalize(), &mut password).unwrap();
    debug!("got password {password}");

    // Logon, get the session key.
    let req = client
        .post(&post_url)
        .header("Origin", format!("http://{}", args.router_host))
        .form(&[
            ("isTest", "false"),
            ("goformId", "LOGIN"),
            ("password", password),
        ])
        .build()?;

    let res = client.execute(req).await?;

    let Logon { result } = res.json::<Logon>().await?;

    info!("successfully logged on code = {result:?}");

    let req = client
        .get(&get_url)
        .query(&[
            ("isTest", "false"),
            ("multi_data", "1"),
            ("cmd", "loginfo,network_provider,network_type,signalbar"),
        ])
        .build()?;

    let x = client.execute(req).await?.text().await?;

    println!("{x:?}");

    Ok(())
}

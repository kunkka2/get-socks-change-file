use std::fs::File;
use std::time::Duration;
use reqwest::Proxy;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    socks_list_url: String,
    get_list_proxy: Option<String>,
    cert_path: String,
}
fn main() {
    println!("Hello, world!");
    let file = File::open("config.yaml").expect("Could not open file");
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values");
    println!("{:?}", config);
}

async fn get_socks_list(config: &Config) -> Result<String,Box<dyn std::error::Error>> {
    let mut builder = reqwest::Client::builder();
    if  !config.get_list_proxy.is_none() {
        let get_list_proxy = config.get_list_proxy.as_ref().unwrap();
        let proxy = Proxy::all(get_list_proxy).expect("Could not get all proxies");
        builder = builder.proxy(proxy);
    }
    builder = builder.timeout(Duration::from_secs(3));
    let client = builder.build().expect("Could not connect to server");
    let data = client.get(&config.socks_list_url).send().await.expect("Could not get socks list");
    let text = data.text().await.expect("Could not read content");
    Ok(text)
}
use std::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    socks_list_url: String,
    get_list_proxy: String,
    cert_path: String,
}
fn main() {
    println!("Hello, world!");
    let file = File::open("config.yaml").expect("Could not open file");
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values");
    println!("{:?}", config);
}

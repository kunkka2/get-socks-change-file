use std::fs::File;
use std::time::Duration;
use reqwest::Proxy;
use serde::{Serialize, Deserialize};
use tokio::{runtime, task};
use futures::future::join_all;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct ProxySource {
    socks_list_url: String,
    get_list_proxy: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    proxy_sources: Vec<ProxySource>,
    cert_path: String,
}
fn main() {
    println!("Hello, world!");
    let file = File::open("config.yaml").expect("Could not open file");
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values");
    println!("{:?}", config);
    runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mut handles =Vec::new();
            for oneproxy in &config.proxy_sources {
                let handle = task::spawn(get_socks_list(oneproxy.clone()));
                handles.push(handle);
                //let handle = task.spawn(get_socks_list((oneproxy)));

                // let res = get_socks_list(oneproxy).await;
                // if res.is_ok() {
                //     print!("{}",res.unwrap());
                // }
            }
            let all_res = join_all(handles).await;
            for res in all_res {
                if res.is_ok(){
                    let txt = res.unwrap();
                    println!("{}",txt);
                }
            }
            println!("still use conifg {}",config.proxy_sources.len());
        })
}

async fn get_socks_list(oneproxy: ProxySource) -> String {
    let mut builder = reqwest::Client::builder();
    if  !oneproxy.get_list_proxy.is_none() {
        let get_list_proxy = oneproxy.get_list_proxy.as_ref().unwrap();
        let proxy = Proxy::all(get_list_proxy).expect("Could not get all proxies");
        builder = builder.proxy(proxy);
    }
    builder = builder.timeout(Duration::from_secs(3));
    let client = builder.build().expect("Could not connect to server");
    let data = client.get(&oneproxy.socks_list_url).send().await.expect("Could not get socks list");
    let text = data.text().await.expect("Could not read content");
    text
}
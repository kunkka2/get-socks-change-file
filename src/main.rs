
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

async fn main_task(){
    let file = File::open("config.yaml").expect("Could not open file");
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values");
    println!("{:?}", config);
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

    let mut data = "".to_string();
    for res in all_res {
        if res.is_ok(){
            let txt = res.unwrap();
            println!("{}",txt);
            if txt.len() > 5 {
                data = data.trim().to_string();
                data += "\n";
                data += &txt;
            }

        }
    }
    println!("still use conifg {}",config.proxy_sources.len());
    println!("socks list is: {}", data);
}
fn main() {

    runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            main_task().await;
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


#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::net::{SocketAddr};
    use hyper::server::conn::http1;
    use hyper::{Request, Response};
    // #[allow(unused_imports)]
    // use hyper::body::Bytes;
    use tokio::task;
    use tokio::net::TcpListener;
    use hyper_util::rt::tokio::TokioIo;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioTimer;
    use futures::future::try_select;
    use super::*;


    const USE_PROXY:&str = "47.238.205.61:8888";
    async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<String>, Infallible> {
        println!("request hello method");
        Ok(Response::new(format!("{}\n{}",USE_PROXY,USE_PROXY)))
    }

    #[tokio::test]
    async  fn test_main_run(){
        let addr = SocketAddr::from(([127, 0, 0, 1], 1081));
        let listener = TcpListener::bind(addr).await.expect("Could not bind to address");
        let server_task = task::spawn(async move {
            loop {
                let (stream, _)= listener.accept().await.expect("Could not accept connection");
                println!("listen port info 000");
                let io = TokioIo::new(stream);
                // Spawn a tokio task to serve multiple connections concurrently
                tokio::task::spawn(async move {
                    // Finally, we bind the incoming connection to our `hello` service
                    println!("task handle request 001");
                    if let Err(err) = http1::Builder::new()
                        .timer(TokioTimer::new())
                        // `service_fn` converts our function in a `Service`
                        .serve_connection(io, service_fn(hello))
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }

        });
        let _ = try_select(server_task,tokio::task::spawn(async {
            println!("start time11!");
            main_task().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(65)).await;
            println!("end time11!");
        })).await;
    }
}

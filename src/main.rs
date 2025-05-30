
use std::fs::File;
use std::time::Duration;
use std::sync::Arc;
use reqwest::Proxy;
use serde::{Serialize, Deserialize};
use tokio::{
    runtime,
    task,
    sync::{Mutex}
};
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


struct Ipres {
    usetime:Duration,
    ipstr:String,
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

    let lines = data.lines();
    let mut filtered_lines= Vec::new();
    for line in lines {
        if !filtered_lines.contains(&line.to_string()) {
            filtered_lines.push(line.to_string());
        }
    }
    println!("{:?}", filtered_lines);
    let length = filtered_lines.len();
    let count = length.div_ceil(4);
    let mut handles = Vec::new();
    let works:Arc<Mutex<Vec<Ipres>>>= Arc::new(Mutex::new(Vec::new()));
    let uselines = Arc::new(Mutex::new(filtered_lines));
    let mut startloop = 0;
    loop {

        let works_one = works.clone();
        let worklines = uselines.clone();
        let handle = tokio::spawn(async move {
            let mut end = startloop + count;
            if end > length {
                end = length;
            }
            for i in startloop..end {
                let start = tokio::time::Instant::now();
                tokio::time::sleep(Duration::from_secs(5)).await;
                let linestask = worklines.lock().await;
                let  ipres = Ipres {
                    usetime:start.elapsed(),
                    ipstr: linestask[i].clone(),
                };
                let mut worksc=works_one.lock().await;
                worksc.push(ipres);
            }
        });
        handles.push(handle);
        if startloop + count >= length {
            println!("done loop lineworks");
            break;
        } else {
            println!("still loop lineworks{}:{}:{}",startloop,count,length);
            startloop += count;
        }
    }
    join_all(handles).await;
    let worksc= works.lock().await;
    let apc =worksc.iter().min_by_key(|ipres| ipres.usetime);
    if apc.is_some() {
        println!("{:?}",apc.unwrap().ipstr);
    }

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

    use tokio::{
        task,
        io::{self, AsyncWriteExt},
        net::{
            TcpListener,
            TcpStream,
        }
    };
    use hyper_util::rt::tokio::TokioIo;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioTimer;
    use futures::future::select_all;
    use std::sync::Arc;

    use socks5_server::{
        auth::NoAuth,
        connection::state::NeedAuthenticate,
        proto::{Address, Error, Reply},
        Command, IncomingConnection, Server,
    };

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
        let prox_task = task::spawn(async move {
            let listener = TcpListener::bind("127.0.0.1:1082").await.expect("could not bind to address");
            let auth = Arc::new(NoAuth) as Arc<_>;
            let server = Server::new(listener, auth);
            while let Ok((conn, _)) = server.accept().await {
                tokio::spawn(async move {
                    match handle(conn).await {
                        Ok(()) => {}
                        Err(err) => eprintln!("{err}"),
                    }
                });
            }
        });
        let _ = select_all([server_task,prox_task,tokio::task::spawn(async move{
            println!("start time11! wait 2s; wait server");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            main_task().await;
            //tokio::time::sleep(tokio::time::Duration::from_secs(65)).await;
            println!("end time11!");
        })]).await;
    }
    async fn handle(conn: IncomingConnection<(), NeedAuthenticate>) -> Result<(), Error> {
        let conn = match conn.authenticate().await {
            Ok((conn, _)) => conn,
            Err((err, mut conn)) => {
                let _ = conn.shutdown().await;
                return Err(err);
            }
        };

        match conn.wait().await {
            Ok(Command::Associate(associate, _)) => {
                let replied = associate
                    .reply(Reply::CommandNotSupported, Address::unspecified())
                    .await;

                let mut conn = match replied {
                    Ok(conn) => conn,
                    Err((err, mut conn)) => {
                        let _ = conn.shutdown().await;
                        return Err(Error::Io(err));
                    }
                };

                let _ = conn.close().await;
            }
            Ok(Command::Bind(bind, _)) => {
                let replied = bind
                    .reply(Reply::CommandNotSupported, Address::unspecified())
                    .await;

                let mut conn = match replied {
                    Ok(conn) => conn,
                    Err((err, mut conn)) => {
                        let _ = conn.shutdown().await;
                        return Err(Error::Io(err));
                    }
                };

                let _ = conn.close().await;
            }
            Ok(Command::Connect(connect, addr)) => {
                let target = match addr {
                    Address::DomainAddress(domain, port) => {
                        let domain = String::from_utf8_lossy(&domain);
                        TcpStream::connect((domain.as_ref(), port)).await
                    }
                    Address::SocketAddress(addr) => TcpStream::connect(addr).await,
                };

                if let Ok(mut target) = target {
                    let replied = connect
                        .reply(Reply::Succeeded, Address::unspecified())
                        .await;

                    let mut conn = match replied {
                        Ok(conn) => conn,
                        Err((err, mut conn)) => {
                            let _ = conn.shutdown().await;
                            return Err(Error::Io(err));
                        }
                    };

                    let res = io::copy_bidirectional(&mut target, &mut conn).await;
                    let _ = conn.shutdown().await;
                    let _ = target.shutdown().await;

                    res?;
                } else {
                    let replied = connect
                        .reply(Reply::HostUnreachable, Address::unspecified())
                        .await;

                    let mut conn = match replied {
                        Ok(conn) => conn,
                        Err((err, mut conn)) => {
                            let _ = conn.shutdown().await;
                            return Err(Error::Io(err));
                        }
                    };

                    let _ = conn.shutdown().await;
                }
            }
            Err((err, mut conn)) => {
                let _ = conn.shutdown().await;
                return Err(err);
            }
        }

        Ok(())
    }
}

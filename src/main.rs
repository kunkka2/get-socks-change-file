
use std::fs::File;
use std::time::Duration;
use std::sync::Arc;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct TargetObj {
    target_path: String,
    target_start: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct Config {
    proxy_sources: Vec<ProxySource>,
    check_url: String,
    check_host: String,
    host_dns: String,
    timeout: u64,
    targets: Vec<TargetObj>,
}


struct Ipres {
    usetime:Duration,
    ipstr:String,
}


fn modify_line(file_path: &str, ipok:&str, target_start:&str) -> io::Result<()> {
    let path = Path::new(file_path);

    // Read the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut buffer = String::new();


    // Iterate over each line
    for line_result in reader.lines() {
        let line = line_result?;
        if line.starts_with(target_start) {
            let newline = target_start.to_string() + ipok;
            buffer.push_str(&newline);
            buffer.push('\n');
        } else {
            buffer.push_str(&line);
            buffer.push('\n');
        }

    }

    // Write back to the file
    let mut writer = BufWriter::new(File::create(path)?);
    writer.write_all(buffer.as_bytes())?;
    Ok(())
}


async fn req_check_speed(ipport: &str, config: &Config) -> Result<(), reqwest::Error> {
    let _ = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&[45, 45, 45, 45, 45, 66, 69, 71, 73, 78, 32, 67, 69, 82, 84, 73, 70, 73, 67, 65, 84, 69, 45, 45, 45, 45, 45, 10, 77, 73, 73, 68, 109, 84, 67, 67, 65, 111, 71, 103, 65, 119, 73, 66, 65, 103, 73, 85, 88, 113, 75, 52, 52, 52, 55, 65, 110, 110, 109, 100, 69, 110, 48, 85, 111, 115, 50, 117, 117, 106, 105, 109, 55, 112, 99, 119, 68, 81, 89, 74, 75, 111, 90, 73, 104, 118, 99, 78, 65, 81, 69, 76, 10, 66, 81, 65, 119, 88, 68, 69, 76, 77, 65, 107, 71, 65, 49, 85, 69, 66, 104, 77, 67, 86, 86, 77, 120, 68, 122, 65, 78, 66, 103, 78, 86, 66, 65, 103, 77, 66, 107, 82, 108, 98, 109, 108, 104, 98, 68, 69, 79, 77, 65, 119, 71, 65, 49, 85, 69, 66, 119, 119, 70, 82, 87, 70, 121, 10, 100, 71, 103, 120, 68, 106, 65, 77, 66, 103, 78, 86, 66, 65, 111, 77, 66, 85, 70, 48, 90, 88, 78, 48, 77, 82, 119, 119, 71, 103, 89, 68, 86, 81, 81, 68, 68, 66, 78, 121, 98, 50, 57, 48, 88, 48, 78, 66, 88, 50, 90, 118, 99, 108, 57, 109, 97, 88, 74, 108, 90, 109, 57, 52, 10, 77, 66, 52, 88, 68, 84, 73, 49, 77, 68, 69, 121, 78, 68, 65, 53, 77, 106, 103, 48, 77, 49, 111, 88, 68, 84, 73, 51, 77, 84, 65, 121, 77, 84, 65, 53, 77, 106, 103, 48, 77, 49, 111, 119, 88, 68, 69, 76, 77, 65, 107, 71, 65, 49, 85, 69, 66, 104, 77, 67, 86, 86, 77, 120, 10, 68, 122, 65, 78, 66, 103, 78, 86, 66, 65, 103, 77, 66, 107, 82, 108, 98, 109, 108, 104, 98, 68, 69, 79, 77, 65, 119, 71, 65, 49, 85, 69, 66, 119, 119, 70, 82, 87, 70, 121, 100, 71, 103, 120, 68, 106, 65, 77, 66, 103, 78, 86, 66, 65, 111, 77, 66, 85, 70, 48, 90, 88, 78, 48, 10, 77, 82, 119, 119, 71, 103, 89, 68, 86, 81, 81, 68, 68, 66, 78, 121, 98, 50, 57, 48, 88, 48, 78, 66, 88, 50, 90, 118, 99, 108, 57, 109, 97, 88, 74, 108, 90, 109, 57, 52, 77, 73, 73, 66, 73, 106, 65, 78, 66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 69, 70, 10, 65, 65, 79, 67, 65, 81, 56, 65, 77, 73, 73, 66, 67, 103, 75, 67, 65, 81, 69, 65, 116, 52, 85, 101, 114, 104, 105, 75, 48, 66, 86, 104, 66, 98, 67, 56, 69, 122, 99, 50, 69, 72, 56, 99, 57, 107, 103, 85, 43, 81, 108, 119, 73, 72, 65, 122, 74, 78, 113, 97, 86, 105, 67, 107, 10, 49, 56, 52, 79, 49, 84, 111, 70, 110, 73, 112, 103, 107, 85, 85, 82, 113, 82, 67, 109, 97, 50, 68, 75, 75, 114, 116, 103, 83, 119, 88, 102, 57, 120, 89, 99, 86, 50, 50, 103, 73, 55, 110, 48, 43, 119, 83, 68, 52, 79, 54, 79, 50, 113, 97, 52, 100, 85, 85, 119, 105, 71, 43, 119, 10, 49, 82, 74, 72, 121, 74, 53, 86, 90, 88, 72, 119, 48, 112, 102, 117, 121, 43, 68, 74, 68, 55, 81, 76, 100, 118, 65, 102, 47, 72, 72, 72, 111, 113, 78, 48, 85, 82, 110, 87, 88, 55, 109, 109, 102, 47, 116, 106, 51, 105, 108, 75, 86, 118, 90, 90, 77, 78, 74, 50, 47, 110, 47, 100, 10, 73, 121, 54, 48, 43, 55, 116, 50, 108, 102, 100, 54, 51, 53, 74, 54, 74, 105, 57, 86, 54, 54, 83, 113, 109, 109, 122, 66, 75, 76, 85, 105, 52, 109, 112, 112, 68, 105, 99, 50, 76, 97, 68, 43, 56, 116, 49, 114, 49, 100, 79, 113, 83, 66, 49, 47, 77, 77, 119, 56, 72, 121, 79, 53, 10, 85, 43, 107, 98, 83, 102, 107, 89, 104, 118, 98, 90, 57, 117, 78, 43, 81, 103, 85, 88, 73, 74, 119, 120, 103, 120, 48, 77, 108, 118, 98, 118, 122, 83, 49, 101, 57, 55, 108, 79, 83, 110, 68, 70, 122, 121, 48, 98, 101, 98, 71, 67, 56, 52, 108, 66, 86, 102, 54, 70, 103, 54, 115, 102, 10, 88, 75, 74, 110, 49, 73, 113, 100, 99, 81, 74, 57, 78, 49, 101, 85, 107, 83, 49, 112, 75, 54, 53, 74, 75, 106, 51, 83, 112, 108, 49, 122, 43, 110, 118, 100, 98, 97, 112, 100, 69, 119, 73, 68, 65, 81, 65, 66, 111, 49, 77, 119, 85, 84, 65, 100, 66, 103, 78, 86, 72, 81, 52, 69, 10, 70, 103, 81, 85, 75, 121, 104, 57, 119, 86, 51, 75, 110, 106, 101, 117, 108, 89, 70, 75, 78, 51, 84, 55, 71, 83, 89, 75, 51, 113, 89, 119, 72, 119, 89, 68, 86, 82, 48, 106, 66, 66, 103, 119, 70, 111, 65, 85, 75, 121, 104, 57, 119, 86, 51, 75, 110, 106, 101, 117, 108, 89, 70, 75, 10, 78, 51, 84, 55, 71, 83, 89, 75, 51, 113, 89, 119, 68, 119, 89, 68, 86, 82, 48, 84, 65, 81, 72, 47, 66, 65, 85, 119, 65, 119, 69, 66, 47, 122, 65, 78, 66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 115, 70, 65, 65, 79, 67, 65, 81, 69, 65, 75, 73, 75, 113, 10, 89, 103, 116, 70, 105, 82, 89, 69, 103, 122, 49, 114, 118, 67, 55, 68, 87, 47, 110, 43, 119, 109, 50, 84, 49, 109, 122, 57, 104, 49, 50, 79, 103, 55, 104, 51, 71, 76, 88, 90, 47, 80, 108, 88, 116, 101, 84, 81, 85, 117, 53, 117, 81, 57, 73, 56, 66, 120, 85, 100, 122, 51, 99, 72, 10, 79, 82, 107, 74, 78, 76, 105, 47, 108, 113, 112, 49, 48, 115, 107, 86, 87, 67, 47, 102, 115, 121, 50, 74, 66, 71, 107, 106, 114, 78, 109, 101, 121, 83, 56, 77, 101, 53, 55, 57, 116, 111, 52, 47, 102, 103, 109, 82, 104, 107, 83, 75, 112, 54, 73, 78, 100, 118, 74, 71, 97, 90, 50, 112, 10, 90, 65, 65, 100, 112, 97, 78, 74, 69, 80, 75, 112, 70, 98, 108, 52, 57, 48, 100, 116, 73, 90, 113, 89, 75, 109, 86, 115, 85, 78, 107, 108, 85, 53, 110, 77, 101, 67, 47, 55, 83, 73, 97, 104, 74, 89, 105, 70, 83, 75, 55, 67, 109, 51, 105, 66, 72, 90, 119, 104, 109, 100, 103, 74, 10, 68, 86, 89, 66, 81, 43, 104, 101, 114, 114, 47, 85, 108, 85, 87, 75, 98, 69, 105, 85, 50, 108, 53, 90, 49, 71, 54, 116, 72, 104, 120, 102, 87, 105, 57, 121, 117, 57, 53, 115, 90, 77, 110, 102, 51, 90, 116, 107, 84, 105, 77, 70, 115, 121, 73, 47, 115, 47, 69, 114, 110, 71, 52, 79, 10, 82, 85, 56, 100, 87, 88, 117, 85, 88, 43, 84, 110, 99, 80, 98, 75, 72, 117, 82, 71, 113, 47, 120, 84, 50, 71, 47, 53, 74, 122, 98, 117, 83, 102, 87, 98, 43, 90, 74, 90, 77, 50, 70, 111, 50, 117, 107, 72, 113, 55, 98, 100, 119, 86, 112, 119, 110, 100, 52, 106, 80, 98, 77, 118, 10, 56, 107, 103, 109, 80, 100, 120, 117, 54, 51, 115, 70, 82, 103, 114, 73, 116, 81, 61, 61, 10, 45, 45, 45, 45, 45, 69, 78, 68, 32, 67, 69, 82, 84, 73, 70, 73, 67, 65, 84, 69, 45, 45, 45, 45, 45, 10])?)
        .resolve(&config.check_host, config.host_dns.parse().unwrap_or("127.0.0.1:443".parse().unwrap()))
        .proxy(Proxy::all("SOCKS5://".to_string() + ipport)?)
        .timeout(Duration::from_secs(config.timeout))
        .build()?
        .get(&config.check_url)
        .send()
        .await?;
    Ok(())
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

    let lines = data.lines();
    let mut filtered_lines= Vec::new();
    for line in lines {
        if !line.is_empty() && !filtered_lines.contains(&line.to_string()) {
            filtered_lines.push(line.to_string());
        }
    }
    println!("filtered_lines is :\n{:?}", filtered_lines);
    let length = filtered_lines.len();
    let count = length.div_ceil(4);
    let mut handles = Vec::new();
    let works:Arc<Mutex<Vec<Ipres>>>= Arc::new(Mutex::new(Vec::new()));
    let config_arc =  Arc::new(config.clone());// No need for Mutex if config is read-only
    let uselines = Arc::new(filtered_lines);// No need for Mutex if config is read-only
    let mut startloop = 0;
    loop {

        let works_one = works.clone();
        let linestask = uselines.clone();
        let config_clone  = config_arc.clone();
        let handle = tokio::spawn(async move {
            let mut end = startloop + count;
            if end > length {
                end = length;
            }
            for i in startloop..end {
                let start = tokio::time::Instant::now();


                let  mut ipres = Ipres {
                    usetime:start.elapsed(),
                    ipstr: linestask[i].clone(),
                };
                let mut flag = false;
                #[cfg(not(test))]{
                    let res = req_check_speed(&ipres.ipstr,&config_clone).await;
                    if res.is_ok(){
                        ipres.usetime = start.elapsed();
                        println!("..........{}",ipres.ipstr);
                        flag = true;
                    }
                }
                #[cfg(test)]
                {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    ipres.usetime = start.elapsed();
                    flag = true;
                }
                if flag {
                    let mut worksc=works_one.lock().await;
                    worksc.push(ipres);
                }


            }
        });
        handles.push(handle);
        if startloop + count >= length {
            println!("done loop lineworks:{}:{}:{}",startloop,count,length);
            break;
        } else {
            println!("still loop lineworks:{}:{}:{}",startloop,count,length);
            startloop += count;
        }
    }
    join_all(handles).await;
    let worksc= works.lock().await;
    let apc =worksc.iter().min_by_key(|ipres| ipres.usetime);
    if apc.is_some() {
        let apcres = apc.unwrap();
        println!("use {:?} to pass request",apcres.ipstr);
        for target in config.targets {
            let target_file = target.target_path.clone();
            let target_start = target.target_start.clone();
            match modify_line(&target_file,&apcres.ipstr,&target_start) {
                Ok(_) => println!("File modified successfully!"),
                Err(e) => eprintln!("Error modifying file: {}", e),
            }
        }

    } else {
        println!(" what happened?");
    }

}

fn main() {

    runtime::Builder::new_multi_thread()
        .worker_threads(4)
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


    const USE_PROXY:&str = "47.238.205.61:8888\n47.238.205.62:8888\n47.238.205.63:8888\n47.238.205.64:8888\n47.238.205.65:8888\n47.238.205.66:8888\n47.238.205.67:8888\n47.238.205.68:8888\n47.238.205.69:8888\n47.238.205.70:8888\n47.238.205.71:8888\n47.238.205.72:8888\n47.238.205.73:8888\n47.238.205.74:8888\n47.238.205.75:8888\n47.238.205.76:8888\n47.238.205.77:8888\n47.238.205.78:8888\n47.238.205.79:8888\n47.238.205.80:8888\n47.238.205.81:8888\n47.238.205.82:8888\n47.238.205.83:8888\n47.238.205.84:8888";
    async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<String>, Infallible> {
        println!("request hello method");
        Ok(Response::new(format!("{}\n{}",USE_PROXY,USE_PROXY)))
    }


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
            let ipres = Ipres {
              usetime: Duration::from_secs(1),
                ipstr: "127.0.0.1:1082".to_string(),
            };
            let config = Config {
                proxy_sources: Vec::new(),
                check_url: "http://127.0.0.1:1081".to_string(),
                check_host: "localhost".to_string(),
                host_dns: "127.0.0.1:1081".to_string(),
                timeout: 2,
                targets: Vec::new(),
            };
            let res = req_check_speed(&ipres.ipstr,&config).await;
            if res.is_ok() {
                println!("req_check_speed run test check!");
            } else {
                println!("Error: when req_check_speed run test check!");
            }
            //tokio::time::sleep(tokio::time::Duration::from_secs(65)).await;
            println!("end time11!");
        })]).await;
    }
    #[test]
    fn test_go_main() {
        runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                test_main_run().await;
            })
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

use std::net::SocketAddr;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use dashmap::DashMap;

const MAX_REQUESTS_PER_MIN: usize = 60;
const SERVER_PORT: u16 = 25565;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], SERVER_PORT))).await.unwrap();
    println!("Listening on port {}", SERVER_PORT);

    let ip_requests = Arc::new(DashMap::new());

    let (tx, mut rx) = mpsc::channel(1000);
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            ip_requests.clear();
        }
    });

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let ip = addr.ip().to_string();
        let request_count = ip_requests.entry(ip.clone()).or_insert(0);
        let ip_requests_clone = Arc::clone(&ip_requests);
        if *request_count > MAX_REQUESTS_PER_MIN {
            tx.send(ip.clone()).await.unwrap();
        }
        *request_count += 1;
        ip_requests_clone.shrink_to_fit();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break, // EOF, so the connection was closed
                    Ok(n) => {
                        if buf[0] == 0x00 || buf[0] == 0xFE || buf[0] == 0x02 {
                            continue;
                        }
                        if let Err(e) = socket.write_all(&buf[0..n]).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}

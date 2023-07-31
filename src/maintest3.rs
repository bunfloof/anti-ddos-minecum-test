use std::net::SocketAddr;
use std::time::Duration;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, stdin, BufReader};
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;
use dashmap::DashMap;

const MAX_REQUESTS_PER_MIN: usize = 60;
const SERVER_PORT: u16 = 25565;

struct Client {
    stream: TcpStream,
    ip: String,
}

async fn handle_client(mut client: Client, ip_requests: Arc<DashMap<String, usize>>, mut sender: mpsc::Sender<String>) {
    let mut buffer = [0; 1024];
    loop {
        match client.stream.read(&mut buffer).await {
            Ok(bytes) => {
                if bytes == 0 {
                    break;
                }
                let request_count = ip_requests.get_mut(&client.ip).unwrap();
                if *request_count > MAX_REQUESTS_PER_MIN {
                    sender.send(client.ip.clone()).await.unwrap();
                    break;
                }
                *request_count += 1;
            }
            Err(_) => break,
        }
    }
}

async fn handle_console(mut receiver: mpsc::Receiver<String>) {
    let mut banned_ips = DashMap::new();
    loop {
        while let Some(ip) = receiver.recv().await {
            banned_ips.insert(ip.clone(), ());
            println!("Banned IP: {}", ip);
        }
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).await.unwrap();
        let parts: Vec<&str> = buffer.trim().split_whitespace().collect();
        if parts.len() != 2 {
            println!("Invalid command. Usage: [ban/unban] [ip]");
            continue;
        }
        match parts[0] {
            "ban" => {
                banned_ips.insert(parts[1].to_string(), ());
                println!("Banned IP: {}", parts[1]);
            }
            "unban" => {
                banned_ips.remove(parts[1]);
                println!("Unbanned IP: {}", parts[1]);
            }
            _ => println!("Unknown command: {}. Usage: [ban/unban] [ip]", parts[0]),
        }
    }
}

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

    let (banned_tx, banned_rx) = mpsc::channel(1000);
    tokio::spawn(async move {
        handle_console(banned_rx).await;
    });

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let ip = addr.ip().to_string();
        ip_requests.insert(ip.clone(), 0);
        let client = Client { stream: socket, ip };
        let ip_requests = Arc::clone(&ip_requests);
        let banned_tx = banned_tx.clone();
        tokio::spawn(async move {
            handle_client(client, ip_requests, banned_tx).await;
        });
        while let Some(ip) = rx.recv().await {
            ip_requests.remove(&ip);
            println!("Removed ip: {}", ip);
        }
    }
}

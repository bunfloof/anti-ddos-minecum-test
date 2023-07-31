use std::net::SocketAddr;
use std::time::Duration;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, stdin};
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;
use dashmap::DashMap;

const LOG_FILE: &'static str = "/var/log/ddos_defender.log";

const MAX_REQUESTS_PER_MIN: usize = 60;
const SERVER_PORT: u16 = 25565;

struct Client {
    stream: TcpStream,
    ip: String,
}

#[derive(Debug)]
enum Packet {
    Ping,
    Handshake,
    Login,
    Unknown,
}

impl FromStr for Packet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ping" => Ok(Packet::Ping),
            "Handshake" => Ok(Packet::Handshake),
            "Login" => Ok(Packet::Login),
            _ => Ok(Packet::Unknown),
        }
    }
}

struct Logger {
    file: std::fs::File,
}

impl Logger {
    fn new(filename: &str) -> Logger {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)
            .unwrap();

        Logger { file }
    }

    fn log(&mut self, message: &str) {
        writeln!(self.file, "{}", message).unwrap();
    }
}

async fn handle_client(mut client: Client, ip_requests: Arc<DashMap<String, usize>>, mut sender: mpsc::Sender<String>, mut logger: Arc<Mutex<Logger>>) {
    let mut buffer = [0; 1024];
    loop {
        match client.stream.read(&mut buffer).await {
            Ok(bytes) => {
                if bytes == 0 {
                    break;
                }
                let packet = std::str::from_utf8(&buffer[..bytes]).unwrap();
                match Packet::from_str(packet) {
                    Ok(Packet::Ping) => {
                        let log_message = format!("Received a Ping packet from {}", client.ip);
                        logger.lock().await.log(&log_message);
                        continue;
                    }
                    Ok(Packet::Handshake) => {
                        let log_message = format!("Received a Handshake packet from {}", client.ip);
                        logger.lock().await.log(&log_message);
                        continue;
                    }
                    Ok(Packet::Login) => {
                        let log_message = format!("Received a Login packet from {}", client.ip);
                        logger.lock().await.log(&log_message);
                        continue;
                    }
                    _ => (),
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

async fn handle_console(mut receiver: mpsc::Receiver<String>, mut logger: Arc<Mutex<Logger>>) {
    let mut banned_ips = DashMap::new();
    loop {
        while let Some(ip) = receiver.recv().await {
            banned_ips.insert(ip.clone(), ());
            let log_message = format!("Banned IP: {}", ip);
            logger.lock().await.log(&log_message);
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
                let log_message = format!("Banned IP: {}", parts[1]);
                logger.lock().await.log(&log_message);
            }
            "unban" => {
                banned_ips.remove(parts[1]);
                let log_message = format!("Unbanned IP: {}", parts[1]);
                logger.lock().await.log(&log_message);
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
    let logger = Arc::new(Mutex::new(Logger::new(LOG_FILE)));
    let logger_clone = Arc::clone(&logger);
    tokio::spawn(async move {
        handle_console(banned_rx, logger_clone).await;
    });

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let ip = addr.ip().to_string();
        ip_requests.insert(ip.clone(), 0);
        let client = Client { stream: socket, ip };
        let ip_requests = Arc::clone(&ip_requests);
        let banned_tx = banned_tx.clone();
        let logger_clone = Arc::clone(&logger);
        tokio::spawn(async move {
            handle_client(client, ip_requests, banned_tx, logger_clone).await;
        });
        while let Some(ip) = rx.recv().await {
            ip_requests.remove(&ip);
            let log_message = format!("Removed IP: {}", ip);
            logger.lock().await.log(&log_message);
        }
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write, stdin};
use std::thread;

const MAX_PACKET_SIZE: usize = 512;  // maximum size of a packet (bytes)
const MAX_REQUESTS_PER_MIN: u32 = 60;  // max number of requests allowed per minute from a single IP
const MAX_UNIQUE_IPS_PER_MIN: u32 = 1000;  // max num of unique IPs allowed per minute
const SERVER_PORT: u16 = 25565;  // minecraft server port

struct IpRequests {
    count: u32,
    timestamp: u64,
}

fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

fn handle_client(mut stream: TcpStream, ip_requests: Arc<Mutex<HashMap<String, u32>>>, banned_ips: Arc<Mutex<HashSet<String>>>) {
    let ip = stream.peer_addr().unwrap().ip().to_string();
    let mut buffer = [0; 1024];

    let mut ip_requests = ip_requests.lock().unwrap();
    let request_count = ip_requests.entry(ip.clone()).or_insert(0);

    if *request_count > MAX_REQUESTS_PER_MIN {
        let mut banned_ips = banned_ips.lock().unwrap();
        banned_ips.insert(ip.clone());
    }

    let packet_limit = *request_count > PACKET_LIMIT_PER_MIN;
    *request_count += 1;
    drop(ip_requests);

    if packet_limit {
        println!("😠 Client {} is sending too many packets, closing connection", ip);
        return;
    }

    loop {
        match stream.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 {
                    break;
                }

                let packet_id = buffer[0];
                if packet_id == CPACKET_PING || packet_id == CPACKET_HANDSHAKE || packet_id == CPACKET_LOGIN {
                    continue;
                }

            }
            Err(_) => {
                break;
            }
        }
    }
}


fn handle_console(banned_ips: Arc<Mutex<HashSet<String>>>) {
    let mut input = String::new();
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.len() != 2 {
            println!("Invalid command. Usage: [ban/unban] [ip]");
            continue;
        }

        let mut banned_ips = banned_ips.lock().unwrap();
        match parts[0] {
            "ban" => {
                banned_ips.insert(parts[1].to_string());
                println!("Banned IP: {}", parts[1]);
            }
            "unban" => {
                banned_ips.remove(parts[1]);
                println!("Unbanned IP: {}", parts[1]);
            }
            _ => {
                println!("Unknown command: {}. Usage: [ban/unban] [ip]", parts[0]);
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], SERVER_PORT))).unwrap();
    println!("Listening on port {}", SERVER_PORT);

    let ip_requests = Arc::new(Mutex::new(HashMap::new()));
    let banned_ips = Arc::new(Mutex::new(HashSet::new()));
    let unique_ips = Arc::new(Mutex::new(HashSet::new()));

    // spawn thread to handle console input for banning/unbanning IPs
    let banned_ips_clone = Arc::clone(&banned_ips);
    thread::spawn(move || {
        handle_console(banned_ips_clone);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let ip = stream.peer_addr().unwrap().ip().to_string();
                let unique_ips = Arc::clone(&unique_ips);
                let mut unique_ips = unique_ips.lock().unwrap();
                if unique_ips.len() as u32 > MAX_UNIQUE_IPS_PER_MIN {
                    println!("Warning: High number of unique IPs connecting. Potential DDoS attack!");
                } else {
                    unique_ips.insert(ip.clone());
                }
    
                let ip_requests = Arc::clone(&ip_requests);
                let banned_ips = Arc::clone(&banned_ips);
                thread::spawn(move || {
                    handle_client(stream, ip_requests, banned_ips);
                });
            }
            Err(e) => {
                println!("Failed to accept a connection: {:?}", e);
            }
        }
    
        // every minute, clear unique_ips to reset the count for the next minute (do not remove)
        thread::sleep(std::time::Duration::from_secs(60));
        let mut unique_ips = unique_ips.lock().unwrap();
        unique_ips.clear();
    }
}
    
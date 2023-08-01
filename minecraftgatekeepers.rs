use async_std::task;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use futures::stream::StreamExt;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::net::SocketAddr;

const SERVER_PORT: u16 = 25565;
const MAX_REQUESTS_PER_SECOND: u32 = 1000;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(1);

struct Client {
    ip: String,
    stream: TcpStream,
    requests: u32,
    last_active: u64,
}

impl Client {
    fn new(ip: String, stream: TcpStream) -> Self {
        Self {
            ip,
            stream,
            requests: 0,
            last_active: now(),
        }
    }
}

fn now() -> u64 {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

async fn handle_client(mut client: Arc<Mutex<Client>>) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let n = client.lock().unwrap().stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        {
            let mut client = client.lock().unwrap();
            client.requests += 1;
            client.last_active = now();

            let rate_limiter = ratelimit_meter::DirectRateLimiter::<ratelimit_meter::state::GCRA>::per_second(nonzero!(MAX_REQUESTS_PER_SECOND.into()));

            if !rate_limiter.check() {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Rate limit exceeded"));
            }

            // minecraft packet validation (fuck you gecko)
            let packet = parse_packet(&buffer);
            match packet {
                Some(Packet::Handshake(version)) => {
                    if version != EXPECTED_PROTOCOL_VERSION {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid handshake version"));
                    }
                },
                Some(Packet::Login(username)) => {
                    if !is_valid_username(&username) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid username"));
                    }
                },
                _ => ()
            }
        }
    }
    Ok(())
}


async fn cleanup_clients(clients: Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>) {
    loop {
        async_std::task::sleep(CLEANUP_INTERVAL).await;
        let mut clients = clients.lock().unwrap();
        clients.retain(|_, client| {
            let client = client.lock().unwrap();
            let inactive_time = now() - client.last_active;
            let too_many_requests = client.requests > MAX_REQUESTS_PER_SECOND;
            let too_old = inactive_time > CLEANUP_INTERVAL.as_secs();
            !(too_many_requests || too_old)
        });
    }
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], SERVER_PORT))).await?;
    println!("Listening on port {}", SERVER_PORT);

    let clients = Arc::new(Mutex::new(HashMap::new()));

    let clients_for_cleanup = Arc::clone(&clients);
    task::spawn(cleanup_clients(clients_for_cleanup));

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let ip = stream.peer_addr()?.ip().to_string();
        let client = Arc::new(Mutex::new(Client::new(ip.clone(), stream)));
        let clients = Arc::clone(&clients);
        clients.lock().unwrap().insert(ip.clone(), Arc::clone(&client));

        let client_for_handling = Arc::clone(&client);
        task::spawn(handle_client(client_for_handling));
    }

    Ok(())
}

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use tokio::time::delay_for;

pub struct TokenBucket {
    tokens: u32,
    capacity: u32,
    fill_rate: Duration,
    last_fill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, fill_rate: Duration) -> Self {
        Self {
            tokens: capacity,
            capacity,
            fill_rate,
            last_fill: Instant::now(),
        }
    }

    pub fn consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_fill);

        let added_tokens = (duration.as_secs_f32() / self.fill_rate.as_secs_f32()) as u32;
        self.tokens = (self.tokens + added_tokens).min(self.capacity);
        self.last_fill = now;
    }
}

pub struct IpFilter {
    allowlist: HashSet<String>,
    denylist: HashSet<String>,
    reputation_list: HashMap<String, u32>,
}

impl IpFilter {
    pub fn new() -> Self {
        Self {
            allowlist: HashSet::new(),
            denylist: HashSet::new(),
            reputation_list: HashMap::new(),
        }
    }

    pub fn allow(&mut self, ip: String) {
        self.allowlist.insert(ip);
    }

    pub fn block(&mut self, ip: String) {
        self.denylist.insert(ip);
    }

    pub fn is_allowed(&self, ip: &str) -> bool {
        self.allowlist.contains(ip) && !self.denylist.contains(ip)
    }

    pub fn update_reputation(&mut self, ip: String, reputation: u32) {
        self.reputation_list.insert(ip, reputation);
    }

    pub fn get_reputation(&self, ip: &str) -> Option<&u32> {
        self.reputation_list.get(ip)
    }
}

pub struct Server {
    listener: TcpListener,
    ip_filter: Arc<Mutex<IpFilter>>,
    token_bucket: Arc<Mutex<TokenBucket>>,
}

impl Server {
    pub fn new(listener: TcpListener, ip_filter: Arc<Mutex<IpFilter>>, token_bucket: Arc<Mutex<TokenBucket>>) -> Self {
        Self {
            listener,
            ip_filter,
            token_bucket,
        }
    }

    pub async fn run(self) {
        loop {
            let (socket, addr) = self.listener.accept().await.unwrap();
            let ip = addr.ip().to_string();

            if self.ip_filter.lock().unwrap().is_allowed(&ip) {
                let token_bucket = Arc::clone(&self.token_bucket);
                tokio::spawn(async move {
                    let mut buffer = [0; 1024];
                    loop {
                        let len = socket.read(&mut buffer).await.unwrap();
                        if len == 0 {
                            return;
                        }

                        if token_bucket.lock().unwrap().consume(1) {
                            delay_for(Duration::from_secs(1)).await;
                            continue;
                        }

                        // going to complete later cuz i wanna masturbate now
                    }
                });
            }
        }
    }
}

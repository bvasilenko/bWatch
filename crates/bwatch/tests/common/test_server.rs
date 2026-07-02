#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct TestServer {
    base_url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl TestServer {
    pub fn start(status: u16, body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);

        thread::spawn(move || {
            for stream in listener.incoming().take(3) {
                let Ok(mut stream) = stream else { continue };
                let mut buffer = [0_u8; 2048];
                let Ok(bytes) = stream.read(&mut buffer) else {
                    continue;
                };
                let request = String::from_utf8_lossy(&buffer[..bytes]).into_owned();
                if let Some(path) = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                {
                    captured
                        .lock()
                        .expect("requests lock")
                        .push(path.to_owned());
                }
                let response = format!(
                    "HTTP/1.1 {status} Test\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        Self { base_url, requests }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn hit_count(&self, path: &str) -> usize {
        self.requests
            .lock()
            .expect("requests lock")
            .iter()
            .filter(|request_path| request_path.as_str() == path)
            .count()
    }
}

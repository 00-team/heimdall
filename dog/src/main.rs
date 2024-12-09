use std::{
    collections::HashMap,
    env,
    fs::Permissions,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
    time::Instant,
};

use serde::Serialize;
use serde_tuple::Deserialize_tuple;

const API_DUMP: &str = "https://heimdall.00-team.org/api/sites/dump/";
const API_PING: &str = "https://heimdall.00-team.org/api/sites/ping/";

#[derive(Debug, Deserialize_tuple)]
struct Message {
    status: u16,
    upstream_response_time: f64,
}

#[derive(Serialize, Default, Debug)]
struct Status {
    code: u16,
    count: u64,
    max_time: u64,
    min_time: u64,
    total_time: u64,
}

#[derive(Serialize, Default, Debug)]
struct Dump {
    total: u64,
    total_time: u64,
    max_time: u64,
    min_time: u64,
    status: HashMap<String, Status>,
}

macro_rules! evar {
    ($name:literal) => {
        env::var($name).expect(concat!($name, " was not found in env"))
    };
}

fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_path(".env").expect("could not read .env file");

    // let sock_path = "/tmp/heimdall.dog.sock";
    let sock_path = format!(
        "/usr/share/nginx/socks/heimdall.dog.{}.sock",
        evar!("HEIMDALL_SITE")
    );

    let _ = std::fs::remove_file(&sock_path);
    let server = UnixDatagram::bind(&sock_path)?;
    std::fs::set_permissions(&sock_path, Permissions::from_mode(0o777))?;
    // server.set_nonblocking(true)?;
    let mut buf = vec![0u8; 512];
    let mut dump = Dump::default();
    let mut latest_request = Instant::now();
    let mut latest_ping = Instant::now();

    let client = client_init();

    loop {
        if latest_request.elapsed().as_secs() >= 10 && dump.total != 0 {
            let res = client.post(API_DUMP).json(&dump).send().unwrap();
            if res.status() != reqwest::StatusCode::OK {
                println!("err: {:?}", res.json::<serde_json::Value>());
            }
            dump = Dump::default();
            latest_request = Instant::now();
        }

        if latest_ping.elapsed().as_secs() >= 60 {
            client.post(API_PING).body("").send().unwrap();
            latest_ping = Instant::now();
        }

        let size = match server.recv(buf.as_mut_slice()) {
            Ok(s) => s,
            Err(e) => panic!("server recv error: {e}"), // Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
                                                        // _ => unreachable!(),
        };
        match serde_json::from_slice::<Message>(&buf[24..size]) {
            Ok(msg) => {
                let time = (msg.upstream_response_time * 1000.0) as u64;
                dump.total += 1;
                dump.total_time += time;
                if dump.max_time < time {
                    dump.max_time = time;
                }

                if dump.min_time > time || dump.min_time == 0 {
                    dump.min_time = time;
                }

                let sk = msg.status.to_string();
                if let Some(status) = dump.status.get_mut(&sk) {
                    status.count += 1;
                    status.total_time += time;

                    if status.min_time > time || status.min_time == 0 {
                        status.min_time = time;
                    }
                    if status.max_time < time {
                        status.max_time = time;
                    }
                } else {
                    dump.status.insert(
                        sk,
                        Status {
                            code: msg.status,
                            total_time: time,
                            min_time: time,
                            max_time: time,
                            count: 1,
                        },
                    );
                }
            }
            Err(e) => println!(
                "err: {e}\n[{size}]: {}",
                String::from_utf8_lossy(&buf[..size])
            ),
        }
    }
}

fn client_init() -> reqwest::blocking::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "authorization",
        evar!("HEIMDALL_TOKEN").parse().expect("bad token"),
    );
    reqwest::blocking::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("could not build the client")
}

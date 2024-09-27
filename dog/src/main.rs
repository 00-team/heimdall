use std::{
    env,
    fs::Permissions,
    io::ErrorKind,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
    time::Instant,
};

use serde::Serialize;
use serde_tuple::Deserialize_tuple;

const SOCK_PAHT: &'static str = "/usr/share/nginx/socks/heimdall.dog.sock";

#[derive(Debug, Deserialize_tuple)]
struct Message {
    _status: u16,
    upstream_response_time: f64,
}

#[derive(Serialize, Default)]
struct Dump {
    total: u64,
    total_time: u64,
}

fn main() -> std::io::Result<()> {
    dotenvy::from_path(".env").expect("could not read .env file");
    let _ = std::fs::remove_file(SOCK_PAHT);
    let server = UnixDatagram::bind(SOCK_PAHT)?;
    std::fs::set_permissions(SOCK_PAHT, Permissions::from_mode(0o777))?;
    server.set_nonblocking(true)?;
    let mut buf = vec![0u8; 512];
    let mut dump = Dump::default();
    let mut last_update = Instant::now();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "authorization",
        env::var("HEIMDALL_TOKEN")
            .expect(".env: HEIMDALL_TOKEN")
            .parse()
            .expect("invalid header value"),
    );
    let client = reqwest::blocking::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("could not build the client");
    // client.post("hi").header(key, value)

    loop {
        if last_update.elapsed().as_secs() >= 10 {
            let res = client
                .post("https://heimdall.00-team.org/api/sites/dump/")
                .json(&dump)
                .send()
                .expect("could not send the dump");

            println!("res status: {}", res.status());
            dump = Dump::default();
            last_update = Instant::now();
        }

        let size = match server.recv(buf.as_mut_slice()) {
            Ok(s) => s,
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            _ => unreachable!(),
        };
        match serde_json::from_slice::<Message>(&buf[24..size]) {
            Ok(msg) => {
                println!("msg: {msg:#?}");
                dump.total += 1;
                dump.total_time += (msg.upstream_response_time * 1000.0) as u64;
            }
            Err(e) => println!(
                "err: {e}\n[{size}]: {}",
                String::from_utf8_lossy(&buf[..size])
            ),
        }
    }
}

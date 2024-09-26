use std::{
    fs::Permissions,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
};

const SOCK_PAHT: &'static str = "/usr/share/nginx/socks/heimdall.log.sock";

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Message {
    bytes_sent: u32,
    connection: u32,
    connection_requests: u32,
    connections_active: u32,
    connections_waiting: u32,
    content_type: String,
    hostname: String,
    https: String,
    limit_rate: String,
    msec: f64,
    nginx_version: String,
    realip_remote_addr: String,
    remote_addr: String,
    request: String,
    request_completion: String,
    request_length: u32,
    request_method: String,
    request_time: f64,
    request_uri: String,
    scheme: String,
    server_addr: String,
    server_name: String,
    server_port: u16,
    server_protocol: String,
    ssl_protocol: String,
    status: u16,
    upstream_bytes_received: u32,
    upstream_header_time: f64,
    upstream_response_length: u32,
    upstream_response_time: f64,
}

fn main() -> std::io::Result<()> {
    let _ = std::fs::remove_file(SOCK_PAHT);
    let server = UnixDatagram::bind(SOCK_PAHT)?;
    std::fs::set_permissions(SOCK_PAHT, Permissions::from_mode(0o777))?;
    // server.set_nonblocking(true)?;
    let mut buf = vec![0u8; 4096 * 2];
    // let mut buf = String::with_capacity(4096);

    loop {
        let size = server.recv(buf.as_mut_slice())?;
        println!(
            "[{size}] out: {}\n",
            String::from_utf8_lossy(&buf[31..size]) // String::from_utf8(buf[..size].to_vec()).expect("invalid input")
        );
        match serde_json::from_slice::<Message>(&buf[31..size]) {
            Ok(v) => println!("{v:#?}"),
            Err(e) => println!("err: {e}"),
        }
    }

    // let mut buf = String::with_capacity(1024);

    // if let Ok((mut stream, addr)) = server.accept() {
    //     stream.read_to_string(&mut buf)?;
    //     println!("stream: {addr:?}\n{buf}\n\n")
    // } else {
    //     println!("an error happend while accepting")
    // }

    // Ok(())
}

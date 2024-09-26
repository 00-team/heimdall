use std::{
    fs::Permissions,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
};

const SOCK_PAHT: &'static str = "/tmp/heimdall-test.sock";

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Message {
    bytes_sent: u64,
    request_length: u64,
    request_time: f64,
}

fn main() -> std::io::Result<()> {
    std::fs::remove_file(SOCK_PAHT)?;
    let server = UnixDatagram::bind(SOCK_PAHT)?;
    std::fs::set_permissions(SOCK_PAHT, Permissions::from_mode(0o777))?;
    // server.set_nonblocking(true)?;
    let mut buf = vec![0u8; 4096];
    // let mut buf = String::with_capacity(4096);

    loop {
        let size = server.recv(buf.as_mut_slice())?;
        match serde_json::from_slice::<Message>(&buf[31..size]) {
            Ok(v) => println!("{v:?}"),
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

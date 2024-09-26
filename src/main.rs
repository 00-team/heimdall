use std::{
    fs::Permissions,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
};

const SOCK_PAHT: &'static str = "/tmp/heimdall-test.sock";

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Message {
    ancient_browser: String,
    arg_: String,
    args: String,
    binary_remote_addr: String,
    body_bytes_sent: String,
    bytes_sent: String,
    connection: String,
    connection_requests: String,
    connection_time: String,
    connections_active: String,
    connections_reading: String,
    connections_waiting: String,
    connections_writing: String,
    content_length: String,
    content_type: String,
    cookie_: String,
    date_gmt: String,
    date_local: String,
    document_root: String,
    document_uri: String,
    fastcgi_path_info: String,
    fastcgi_script_name: String,
    geoip_area_code: String,
    geoip_city: String,
    geoip_city_continent_code: String,
    geoip_city_country_code: String,
    geoip_city_country_code3: String,
    geoip_city_country_name: String,
    geoip_country_code: String,
    geoip_country_code3: String,
    geoip_country_name: String,
    geoip_dma_code: String,
    geoip_latitude: String,
    geoip_longitude: String,
    geoip_org: String,
    geoip_postal_code: String,
    geoip_region: String,
    geoip_region_name: String,
    gzip_ratio: String,
    host: String,
    hostname: String,
    http2: String,
    http3: String,
    http_: String,
    https: String,
    invalid_referer: String,
    is_args: String,
    limit_conn_status: String,
    limit_rate: String,
    limit_req_status: String,
    modern_browser: String,
    msec: String,
    msie: String,
    nginx_version: String,
    pid: String,
    pipe: String,
    proxy_add_x_forwarded_for: String,
    proxy_host: String,
    proxy_port: String,
    proxy_protocol_addr: String,
    proxy_protocol_port: String,
    proxy_protocol_server_addr: String,
    proxy_protocol_server_port: String,
    proxy_protocol_tlv_: String,
    proxy_protocol_tlv_aws_vpce_id: String,
    proxy_protocol_tlv_azure_pel_id: String,
    proxy_protocol_tlv_gcp_conn_id: String,
    query_string: String,
    realip_remote_addr: String,
    realip_remote_port: String,
    realpath_root: String,
    remote_addr: String,
    remote_port: String,
    remote_user: String,
    request: String,
    request_body: String,
    request_body_file: String,
    request_completion: String,
    request_filename: String,
    request_id: String,
    request_length: String,
    request_method: String,
    request_time: String,
    request_uri: String,
    scheme: String,
    secure_link: String,
    secure_link_expires: String,
    sent_http_: String,
    sent_trailer_: String,
    server_addr: String,
    server_name: String,
    server_port: String,
    server_protocol: String,
    slice_range: String,
    ssl_alpn_protocol: String,
    ssl_cipher: String,
    ssl_ciphers: String,
    ssl_protocol: String,
    ssl_server_name: String,
    ssl_session_id: String,
    ssl_session_reused: String,
    status: String,
    tcpinfo_rtt: String,
    tcpinfo_rttvar: String,
    tcpinfo_snd_cwnd: String,
    tcpinfo_rcv_space: String,
    uid_got: String,
    uid_reset: String,
    uid_set: String,
    upstream_addr: String,
    upstream_bytes_received: String,
    upstream_bytes_sent: String,
    upstream_cache_status: String,
    upstream_connect_time: String,
    upstream_cookie_: String,
    upstream_header_time: String,
    upstream_http_: String,
    upstream_response_length: String,
    upstream_response_time: String,
    upstream_status: String,
    upstream_trailer_: String,
    uri: String,
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
        // println!(
        //     "[{size}] out: {}",
        //     String::from_utf8(buf[31..size].to_vec()).expect("invalid input")
        // );
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

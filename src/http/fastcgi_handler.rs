use std::net::TcpStream;
use std::net::SocketAddr;

use fastcgi_client::Client;
use fastcgi_client::Params;
use http::Request;
use hyper::header::HeaderValue;
use hyper::header::HeaderName;
use hyper::Response;
use hyper::Body;
use regex::Regex;
use regex::Captures;
use std::collections::HashMap;

pub(crate) async fn handle_fastcgi(
    document_root: String,
    script_filename: String,
    remote_addr: SocketAddr,
    req: Request<Body>,
    http_port: u16,
    php_port: u16,
) -> anyhow::Result<Response<Body>> {
    let remote_addr = remote_addr.ip().to_string();
    let remote_addr = remote_addr.as_str();
    let request_uri = req.uri().to_string();
    let headers = req.headers().clone();
    let method = req.method().to_string();
    let method = method.as_str();
    let (parts, request_body) = req.into_parts();

    let body = hyper::body::to_bytes(request_body).await.unwrap();

    let http_version = crate::http::version::as_str(parts.version);

    let stream = TcpStream::connect(("127.0.0.1", php_port)).unwrap();
    let mut client = Client::new(stream, false);

    let http_port_str = http_port.to_string();
    let php_port_str = php_port.to_string();

    let empty_header = &HeaderValue::from_str("").unwrap();

    // Fastcgi params, please reference to nginx-php-fpm config.
    let mut params = Params::with_predefine()
        .set_content_length(
            headers
                .get("Content-Length")
                .unwrap_or(empty_header)
                .to_str()
                .unwrap_or(""),
        )
        .set_content_type(
            headers
                .get("Content-Type")
                .unwrap_or(empty_header)
                .to_str()
                .unwrap_or(""),
        )
        .set_document_root(&document_root)
        .set_document_uri(&request_uri)
        .set_query_string("")
        .set_remote_addr(remote_addr)
        .set_remote_port(http_port_str.as_ref())
        .set_request_method(method)
        .set_request_uri(&request_uri)
        .set_script_filename(&script_filename)
        .set_script_name(&request_uri)
        .set_server_addr("127.0.0.1")
        .set_server_name("127.0.0.1")
        .set_server_port(php_port_str.as_ref())
        .set_server_software("rymfony/rust/fastcgi-client")
        .set_server_protocol(http_version);

    let mut headers_normalized = Vec::new();
    for (name, value) in headers.iter() {
        let header_name = format!("HTTP_{}", name.as_str().replace("-", "_").to_uppercase());

        headers_normalized.push((header_name, value.to_str().unwrap()));
    };
    params.extend(headers_normalized.iter().map(|(k, s)| (k.as_str(), *s)));

    let output = client.do_request(&params, &mut std::io::Cursor::new(body)).unwrap();

    let stdout: Vec<u8> = output.get_stdout().unwrap();
    let stdout: &[u8] = stdout.as_slice();
    let stdout: &str = std::str::from_utf8(stdout).unwrap();
    let stdout: String = String::from(stdout);

    let response_headers_regex = Regex::new(r"(?s)^(.*)\r\n\r\n(.*)$").unwrap();

    let capts: Captures = response_headers_regex.captures(&stdout).unwrap();

    let headers: &str = &capts[1];
    let body: String = String::from(&capts[2]);

    let single_header_regex = Regex::new(r"^([^:]+):(.*)$").unwrap();

    let headers_normalized: HashMap<HeaderName, HeaderValue> = headers.split("\r\n").map(|header: &str| {
        let headers_capts = single_header_regex.captures(header).unwrap();

        let header_name = &headers_capts[1].as_bytes();
        let header_value = &headers_capts[2];

        (HeaderName::from_bytes(header_name).unwrap(), HeaderValue::from_str(header_value).unwrap())
    }).collect();

    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.extend(headers_normalized);

    let response = response_builder.body(
        Body::from(body)
        //Body::from(body)
    ).unwrap();

    anyhow::Result::Ok(response)
}

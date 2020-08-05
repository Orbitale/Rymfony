use std::net::SocketAddr;
use std::net::TcpStream;

use fastcgi_client::Client;
use fastcgi_client::Params;
use http::Request;
use hyper::header::HeaderName;
use hyper::header::HeaderValue;
use hyper::Body;
use hyper::Response;
use regex::Captures;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

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
    let uri = req.uri();
    let request_uri = uri.to_string();
    let query_string = uri.query().unwrap_or("").to_string();
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

    let pathinfo = get_pathinfo_from_uri(&request_uri);
    let script_name = if pathinfo.0.len() > 0 {
        let path = PathBuf::from(&document_root).join(pathinfo.0.clone());
        path.to_str().unwrap().to_string()
    } else {
        script_filename.clone()
    };

    // Fastcgi params, please reference to nginx-php-fpm config.
    let mut params = Params::with_predefine();
    params.insert(
        "CONTENT_LENGTH",
        headers
            .get("Content-Length")
            .unwrap_or(empty_header)
            .to_str()
            .unwrap_or(""),
    );
    params.insert(
        "CONTENT_TYPE",
        headers
            .get("Content-Type")
            .unwrap_or(empty_header)
            .to_str()
            .unwrap_or(""),
    );
    params.insert("DOCUMENT_ROOT", &document_root);
    params.insert("DOCUMENT_URI", &request_uri);
    params.insert("PATH_INFO", pathinfo.1.as_str());
    params.insert("QUERY_STRING", &query_string);
    params.insert("REMOTE_ADDR", remote_addr);
    params.insert("REMOTE_PORT", http_port_str.as_ref());
    params.insert("REQUEST_METHOD", method);
    params.insert("REQUEST_URI", &request_uri);
    params.insert("SCRIPT_FILENAME", &script_filename);
    params.insert("SCRIPT_NAME", &script_name);
    params.insert("SERVER_ADDR", "127.0.0.1");
    params.insert("SERVER_NAME", "127.0.0.1");
    params.insert("SERVER_PORT", php_port_str.as_ref());
    params.insert("SERVER_PROTOCOL", http_version);
    params.insert("SERVER_SOFTWARE", "Rymfony v0.1.0");

    let mut headers_normalized = Vec::new();
    for (name, value) in headers.iter() {
        let header_name = format!("HTTP_{}", name.as_str().replace("-", "_").to_uppercase());

        headers_normalized.push((header_name, value.to_str().unwrap()));
    }
    params.extend(headers_normalized.iter().map(|(k, s)| (k.as_str(), *s)));

    let output = client
        .do_request(&params, &mut std::io::Cursor::new(body))
        .unwrap();

    let stdout: Vec<u8> = output.get_stdout().unwrap();
    let stdout: &[u8] = stdout.as_slice();
    let stdout: &str = std::str::from_utf8(stdout).unwrap();
    let stdout: String = String::from(stdout);

    let response_headers_regex = Regex::new(r"(?s)^(.*)\r\n\r\n(.*)$").unwrap();

    let capts: Captures = response_headers_regex.captures(&stdout).unwrap();

    let headers: &str = &capts[1];
    let body: String = String::from(&capts[2]);

    let single_header_regex = Regex::new(r"^([^:]+):(.*)$").unwrap();

    let headers_normalized: HashMap<HeaderName, HeaderValue> = headers
        .split("\r\n")
        .map(|header: &str| {
            let headers_capts = single_header_regex.captures(header).unwrap();

            let header_name = &headers_capts[1].as_bytes();
            let header_value = &headers_capts[2];

            (
                HeaderName::from_bytes(header_name).unwrap(),
                HeaderValue::from_str(header_value).unwrap(),
            )
        })
        .collect();

    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.extend(headers_normalized);

    let response = response_builder
        .body(
            Body::from(body), //Body::from(body)
        )
        .unwrap();

    anyhow::Result::Ok(response)
}

fn get_pathinfo_from_uri(request_uri: &str) -> (String, String) {
    let php_file_regex = Regex::new(r"(^.*\.php)((?:/|$).*)$").unwrap();

    if !php_file_regex.is_match(request_uri) {
        return (String::from(""), request_uri.to_string());
    }

    let capts: Captures = php_file_regex.captures(request_uri).unwrap();

    let php_file = capts[1].trim_start_matches("/").to_string();
    let path_info = capts[2].to_string();

    (php_file, path_info)
}

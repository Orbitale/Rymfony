use std::net::SocketAddr;
use std::net::TcpStream;

use async_fcgi::client::connection::Connection;
use fastcgi_client::Client;
use fastcgi_client::Params;
use bytes::Bytes;
use http::Request;
use hyper::header::HeaderName;
use hyper::header::HeaderValue;
use hyper::Body;
use hyper::Response;
use regex::Captures;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use async_fcgi::FCGIAddr;
use std::io::Error;

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

    let http_version = crate::http::version::as_str(parts.version);

    let stream = match TcpStream::connect(("127.0.0.1", php_port)) {
        Ok(t) => t,
        Err(e) => {
            return anyhow::Result::Ok(error_as_response(e));
        }
    };

    let mut client = Client::new(stream, false);

    let http_port_str = http_port.to_string();
    let php_port_str = php_port.to_string();

    let empty_header = &HeaderValue::from_str("").unwrap();

    let (php_file, pathinfo) = get_pathinfo_from_uri(&request_uri);
    let script_name = if pathinfo.len() > 0 {
        let path = PathBuf::from(&document_root).join(pathinfo.clone());
        path.to_str().unwrap().to_string()
    } else {
        script_filename.clone()
    };













    // "async-fcgi" attempt
    // Not working, obviously.
    //
    //
    //let fpm_upstream_add = FCGIAddr::from(SocketAddr::from(([127, 0, 0, 1], php_port)));
    //let mut fcgi_con = Connection::connect(&fpm_upstream_add, 1).await.unwrap();
    //let req = Request::get(&pathinfo)
    //    .body(request_body)
    //    .unwrap()
    //;
    //
    //let mut params = HashMap::new();
    //params.insert(
    //    Bytes::from(&b"CONTENT_LENGTH"[..]),
    //    Bytes::from(headers
    //        .get("Content-Length")
    //        .unwrap_or(empty_header)
    //        .to_str()
    //        .unwrap_or("")),
    //);
    //params.insert(
    //    "CONTENT_TYPE".as_bytes(),
    //    headers
    //        .get("Content-Type")
    //        .unwrap_or(empty_header)
    //        .to_str()
    //        .unwrap_or("")
    //        .as_bytes(),
    //);
    //params.insert("GATEWAY_INTERFACE".as_bytes(), "FastCGI/1.0".as_bytes());
    //params.insert("SERVER_SOFTWARE".as_bytes(), "rust/rymfony/fastcgi-client".parse().unwrap());
    //params.insert("SERVER_PROTOCOL".as_bytes(), "HTTP/1.1".parse().unwrap());
    //params.insert("DOCUMENT_ROOT".as_bytes(), &document_root.parse().unwrap());
    //params.insert("DOCUMENT_URI".as_bytes(), &request_uri.parse().unwrap());
    //params.insert("PATH_INFO".as_bytes(), php_file.as_str().parse().unwrap());
    //params.insert("QUERY_STRING".as_bytes(), &query_string.parse().unwrap());
    //params.insert("REMOTE_ADDR".as_bytes(), remote_addr.parse().unwrap());
    //params.insert("REMOTE_PORT".as_bytes(), &[http_port]);
    //params.insert("REQUEST_METHOD".as_bytes(), method.parse().unwrap());
    //params.insert("REQUEST_URI".as_bytes(), &request_uri.parse().unwrap());
    //params.insert("SCRIPT_FILENAME".as_bytes(), &script_filename.parse().unwrap());
    //params.insert("SCRIPT_NAME".as_bytes(), &script_name.parse().unwrap());
    //params.insert("SERVER_ADDR".as_bytes(), "127.0.0.1".parse().unwrap());
    //params.insert("SERVER_NAME".as_bytes(), "127.0.0.1".parse().unwrap());
    //params.insert("SERVER_PORT".as_bytes(), &[php_port]);
    //params.insert("SERVER_PROTOCOL".as_bytes(), http_version.parse().unwrap());
    //params.insert("SERVER_SOFTWARE".as_bytes(), "Rymfony v0.1.0".parse().unwrap());
    
    //let mut res = fcgi_con
    //    .forward(req, params)
    //    .await
    //    .unwrap()
    //;
    //let response_body = res.body();













    //
    // Previous way of doing it: parse the HTTP request raw
    // and hope it's fine.
    // (spoiler alert: it's not)
    //

    
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
    params.insert("PATH_INFO", pathinfo.as_str());
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

    let body_bytes = hyper::body::to_bytes(request_body).await.unwrap();
    let output = client.do_request(&params, &mut std::io::Cursor::new(body_bytes)).unwrap();

    let stderr: Vec<u8> = output.get_stderr().unwrap_or(Vec::new());
    let stdout: Vec<u8> = output.get_stdout().unwrap_or(Vec::new());

    trace!("Received FastCGI response.");
    trace!("STDERR:\n{}", std::str::from_utf8(&stderr).unwrap_or(""));
    trace!("STDOUT:\n{}", std::str::from_utf8(&stdout).unwrap_or(""));

    if stderr.len() > 0 {
        error!("FastCGI returned an error:\n{}", std::str::from_utf8(&stderr).unwrap());
    }




















    //
    // This one is an attempt to convert the contents of "stdout"
    // into a proper Headers list + body as bytes.
    // It's a bit hard to understand how to use the "httparse" library for me,
    // so let's leave it here for now...
    //
    //
    //
    // let mut headers: [HeaderIndex; MAX_HEADERS] = EMPTY_HEADER_INDEX_ARRAY;
    //
    // let (len, ver, status, h_len) = {
    //     let mut parsed: [httparse::Header<'_>; MAX_HEADERS] = EMPTY_HEADER_ARRAY;
    //
    //     let mut res = httparse::Response::new(&mut parsed);
    //     match res.parse(src)? {
    //         httparse::Status::Complete(len) => {
    //             let version = if res.version.unwrap() == 1 {
    //                 Version::HTTP_11
    //             } else {
    //                 Version::HTTP_10
    //             };
    //             let status = StatusCode::from_u16(res.code.unwrap())
    //                 .map_err(|_| ParseError::Status)?;
    //             HeaderIndex::record(src, res.headers, &mut headers);
    //
    //             (len, version, status, res.headers.len())
    //         }
    //         httparse::Status::Partial => return Ok(None),
    //     }
    // };
    //









    // 
    // Here you can admire an attempt to parse response headers using the "http_bytes" library.
    // Total failure, once again.
    // Admire.
    //
    //let stdout_slice: &[u8] = stdout.as_slice();
    //let mut headers_buffer = vec![http_bytes::EMPTY_HEADER; 100];
    //let (raw_response, body) = match http_bytes::parse_response_header(&stdout_slice.clone(), &mut headers_buffer) {
    //    Ok(t) => {
    //        match t {
    //            Some((response, body)) => {
    //                (response, body)
    //            },
    //            None => {
    //                panic!("Empty response?");
    //            }
    //        }
    //    },
    //    Err(e) => {
    //        error!("An error occured: <<{}>>", e);
    //        let mut resp = http_bytes::http::Response::builder()
    //            .status(502)
    //            .header("Content-Type", "text/plain")
    //            .body(())
    //            .unwrap()
    //        ;
    //        (resp, "A FastCGI error was encountered.".as_bytes())
    //    }
    //};


    
    //
    // Here, you can enjoy seeing me trying to parse an HTTP response...
    // ...with a regex...
    // Enjoy.
    //
    
    let stdout: Vec<u8> = output.get_stdout().unwrap();
    let stdout: &[u8] = stdout.as_slice();
    let stdout: &str = std::str::from_utf8(stdout).unwrap();
    let stdout: String = String::from(stdout);
    let response_headers_regex = Regex::new(r"(?s)^(.*)\r\n\r\n(.*)$").unwrap();
    let single_header_regex = Regex::new(r"^([^:]+):(.*)$").unwrap();
    let capts: Captures = response_headers_regex.captures(&stdout).unwrap();
    let headers: &str = &capts[1];
    let body: String = String::from(&capts[2]);
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
    let response_body = Body::from(body);





    //
    // Mandatory here: a Hyper response.
    // Everything has to be done to convert the fastcgi response into a Hyper response.
    //
    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.extend(headers_normalized);
   
    // let status = raw_response.status();
   
    let response = response_builder
        // .status(status.as_u16())
        .body(
            response_body,
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

fn error_as_response<T>(error: T) -> Response<Body>
where T: std::fmt::Display {
    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.append("Content-Type", "text/html".parse().unwrap());

    let mut body_str = String::from("<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Internal 500 error</title></head><body>Internal 500 Error");
    body_str.push_str(format!("Returned error: <pre>{}</pre>", &error).as_str());
    body_str.push_str("</body></html>");

    let response = response_builder
        .status(500)
        .body(
            Body::from(body_str), //Body::from(body)
        )
        .unwrap();

    response
}

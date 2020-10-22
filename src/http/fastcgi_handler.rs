use std::net::SocketAddr;
use std::net::TcpStream;

use fastcgi_client::Client;
use fastcgi_client::Params;
use http::Request;
use hyper::header::HeaderName;
use hyper::header::HeaderValue;
use hyper::Body;
use hyper::HeaderMap;
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
    let request_headers = req.headers().clone();
    let method = req.method().to_string();
    let method = method.as_str();
    let (parts, request_body) = req.into_parts();

    let http_version = crate::http::version::as_str(parts.version);

    let stream = match TcpStream::connect(("127.0.0.1", php_port)) {
        Ok(t) => t,
        Err(e) => {
            return anyhow::Result::Ok(error_as_response(e, 503));
        }
    };

    let mut client = Client::new(stream, false);

    let http_port_str = http_port.to_string();
    let php_port_str = php_port.to_string();

    let (php_file, pathinfo) = get_pathinfo_from_uri(&request_uri);
    let script_name = if php_file.len() > 0 {
        let path = PathBuf::from(&document_root).join(php_file.clone());
        path.to_str().unwrap().to_string()
    } else {
        script_filename.clone()
    };

    //
    // Mandatory FastCGI parameters.
    // See: https://www.nginx.com/resources/wiki/start/topics/examples/phpfcgi/
    //
    let mut fcgi_params = Params::with_predefine();
    let empty_header = &HeaderValue::from_str("").unwrap();
    fcgi_params.insert("CONTENT_LENGTH", get_header_value(&request_headers, "Content-Length", &empty_header));
    fcgi_params.insert("CONTENT_TYPE", get_header_value(&request_headers, "Content-Type", &empty_header));
    fcgi_params.insert("DOCUMENT_ROOT", &document_root);
    fcgi_params.insert("DOCUMENT_URI", &request_uri);
    fcgi_params.insert("PATH_INFO", pathinfo.as_str());
    fcgi_params.insert("QUERY_STRING", &query_string);
    fcgi_params.insert("REMOTE_ADDR", remote_addr);
    fcgi_params.insert("REMOTE_PORT", http_port_str.as_ref());
    fcgi_params.insert("REQUEST_METHOD", method);
    fcgi_params.insert("REQUEST_URI", &request_uri);
    fcgi_params.insert("SCRIPT_FILENAME", &script_filename);
    fcgi_params.insert("SCRIPT_NAME", &script_name);
    fcgi_params.insert("SERVER_ADDR", "127.0.0.1");
    fcgi_params.insert("SERVER_NAME", "127.0.0.1");
    fcgi_params.insert("SERVER_PORT", php_port_str.as_ref());
    fcgi_params.insert("SERVER_PROTOCOL", http_version);
    fcgi_params.insert("SERVER_SOFTWARE", "Rymfony v0.1.0");

    //
    // Send all Request HTTP headers to FastCGI,
    // in the form of "HTTP_..." parameters.
    // That's supposed to be how FastCGI and PHP work.
    //
    let mut fcgi_headers_normalized = Vec::new();
    for (name, value) in request_headers.iter() {
        let header_name = format!("HTTP_{}", name.as_str().replace("-", "_").to_uppercase());

        fcgi_headers_normalized.push((header_name, value.to_str().unwrap()));
    }
    fcgi_params.extend(fcgi_headers_normalized.iter().map(|(k, s)| (k.as_str(), *s)));

    let request_body_bytes = hyper::body::to_bytes(request_body).await.unwrap();
    let mut fcgi_request_body = &mut std::io::Cursor::new(request_body_bytes);

    //
    // Ignition! Do the request!
    //
    let fcgi_output = client.do_request(&fcgi_params, &mut fcgi_request_body);

    // Retrieve request output
    let (raw_fcgi_stdout, fcgi_stderr) = match fcgi_output {
        Ok(fcgi_output) => {
            (
                fcgi_output.get_stdout().unwrap_or_default(),
                fcgi_output.get_stderr().unwrap_or_default()
            )
        },
        Err(e) => {
            return anyhow::Result::Ok(error_as_response(e, 502));
        }
    };

    if raw_fcgi_stdout.len() == 0 {
        error!("FastCGI returned an empty Response:\n{}", std::str::from_utf8(&fcgi_stderr).unwrap());
        return anyhow::Result::Ok(error_as_response(std::str::from_utf8(fcgi_stderr.as_slice()).unwrap(), 502));
    }

    //
    // The CGI response *never* returns the HTTP Status Line.
    // However, the "httparse" crate needs it.
    // So we create a fake one.
    // Later on, this will be overriden by the "Status" header (see below), so it's a fine hack.
    //
    let mut fcgi_stdout: Vec<u8> = format!("{} 200 Ok\r\n", http_version).as_bytes().to_vec();
    fcgi_stdout.extend(raw_fcgi_stdout);

    trace!("Received FastCGI response.");

    if fcgi_stderr.len() > 0 {
        error!("FastCGI returned an error:\n{}", std::str::from_utf8(&fcgi_stderr).unwrap());
        return anyhow::Result::Ok(error_as_response(std::str::from_utf8(fcgi_stderr.as_slice()).unwrap(), 502));
    }

    //
    // Convert the contents of "fcgi_stdout" into a proper list of HTTP Headers.
    // Body is supposed to be a bunch of bytes.
    //
    let mut normalized_headers = [httparse::EMPTY_HEADER; 80];
    let mut res = httparse::Response::new(&mut normalized_headers);
    let headers_len = res.parse(fcgi_stdout.as_slice())?.unwrap();
    let response_headers = res.headers;
    debug!("Response headers ready to normalize");
    let mut headers_normalized: HashMap<HeaderName, HeaderValue> = response_headers
        .iter()
        .map(|header| {
            let header_name = header.name.as_bytes();
            let header_value = std::str::from_utf8(header.value).unwrap();

            debug!("Normalized headers: \"{}: {}\"", std::str::from_utf8(&header_name).unwrap(), &header_value);

            (
                HeaderName::from_bytes(header_name).unwrap(),
                HeaderValue::from_str(header_value).unwrap(),
            )
        })
        .collect();
    debug!("Response headers are now normalized");
    let (_, body) = fcgi_stdout.split_at(headers_len);

    // ... However, it seems I can't just put bytes in the body, and that only String is possible...
    let body = String::from_utf8(body.to_vec()).unwrap();
    let response_body = Body::from(body);

    //
    // CGI's RFC says that the "Status" response header
    // can contain the HTTP Response Status code.
    // It's not explicit whether it should be removed from the end response,
    // but we use ".remove()" to do so, to make sure there is no conflict between
    // the real HTTP Status line and the "Status" header (what a whoopsie it would be anyway...).
    // See: https://tools.ietf.org/html/rfc3875#section-6.3.3
    //
    let response_status_header = headers_normalized.remove(&HeaderName::from_static("status"));

    let status_code: u16 = if let Some(status_header) = response_status_header {
        use std::str::FromStr;
        let status_code_as_string = &status_header.to_str().unwrap().chars().take(3).collect::<String>();
        let status_code = http::StatusCode::from_str(status_code_as_string).unwrap();
        status_code.as_u16()
    } else {
        debug!("Response does not contain the \"Status\" HTTP header");
        200
    };

    //
    // Finally: a Hyper response.
    // Everything has to be done to convert the fastcgi response into a Hyper response.
    //
    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.extend(headers_normalized);

    let response = response_builder
        .status(status_code)
        .body(response_body)
        .unwrap();

    trace!("Finish response");

    anyhow::Result::Ok(response)
}

fn get_header_value<'a>(headers: &'a HeaderMap<HeaderValue>, header_name: &str, empty_header: &'a HeaderValue) -> &'a str {
    headers
        .get(header_name)
        .unwrap_or(empty_header)
        .to_str()
        .unwrap_or_default()
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

fn error_as_response<T>(error: T, status_code: u16) -> Response<Body>
where T: std::fmt::Display {
    let mut response_builder = Response::builder();
    let response_headers = response_builder.headers_mut().unwrap();
    response_headers.append("Content-Type", "text/html".parse().unwrap());

    let mut body_str = String::from("<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Internal 500 error</title></head><body>Internal 500 Error");
    body_str.push_str(format!("Returned error: <pre>{}</pre>", &error).as_str());
    body_str.push_str("</body></html>");

    let response = response_builder
        .status(status_code)
        .body(
            Body::from(body_str), //Body::from(body)
        )
        .unwrap();

    response
}

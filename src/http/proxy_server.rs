use std::env;
use std::net::SocketAddr;
use std::net::TcpStream;

use console::style;
use fastcgi_client::Client;
use fastcgi_client::Params;
use http::Version;
use hyper::header::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use std::convert::Infallible;

#[tokio::main]
pub(crate) async fn start(http_port: u16, php_port: u16) {
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], http_port));

    let make_service = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        async move {
            let remote_addr = remote_addr.clone();
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle(remote_addr.clone(), req, http_port, php_port)
            }))
        }
    });

    let http_server = Server::bind(&addr).serve(make_service);

    println!(
        "Server listening to {}",
        style(format!("http://{}", addr)).cyan()
    );

    http_server.await.unwrap();
}

async fn handle(
    remote_addr: SocketAddr,
    req: Request<Body>,
    http_port: u16,
    php_port: u16,
) -> Result<Response<Body>, Infallible> {
    let remote_addr = remote_addr.ip().to_string();
    let remote_addr = remote_addr.as_str();
    let document_root = env::current_dir().unwrap();
    let script_filename = document_root.join("index.php");
    let script_filename = script_filename.to_str().unwrap();
    let request_uri = req.uri().to_string();
    let headers = req.headers().clone();
    let method = req.method().to_string();
    let method = method.as_str();
    let (parts, mut request_body) = req.into_parts();

    let mut body = hyper::body::to_bytes(request_body).await.unwrap();

    let http_version = match parts.version {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        Version::HTTP_3 => "HTTP/3.0",
        _ => unreachable!(),
    };

    println!("HTTP/{} {} {}", http_version, method, request_uri);

    let stream = TcpStream::connect(("127.0.0.1", php_port)).unwrap();
    let mut client = Client::new(stream, false);

    let http_port_str = http_port.to_string();
    let php_port_str = php_port.to_string();

    let empty_header = &HeaderValue::from_str("").unwrap();

    // Fastcgi params, please reference to nginx-php-fpm config.
    let params = Params::with_predefine()
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
        .set_document_root(document_root.to_str().unwrap())
        .set_document_uri(&request_uri)
        .set_query_string("")
        .set_remote_addr(remote_addr)
        .set_remote_port(http_port_str.as_ref())
        .set_request_method(method)
        .set_request_uri(&request_uri)
        .set_script_filename(script_filename)
        .set_script_name(&request_uri)
        .set_server_addr("127.0.0.1")
        .set_server_name("127.0.0.1")
        .set_server_port(php_port_str.as_ref())
        .set_server_software("rymfony/rust/fastcgi-client")
        .set_server_protocol(http_version);

    let output = client.do_request(&params, &mut body).unwrap();

    let stdout = output.get_stdout();
    let stdout = stdout.unwrap();
    let stdout = String::from_utf8(stdout);

    let resp = Response::builder();

    let stdout = stdout.unwrap();

    println!("Response body: {}", stdout.clone());

    let resp = resp.body(Body::from(stdout.clone())).unwrap();

    Ok(resp)
}

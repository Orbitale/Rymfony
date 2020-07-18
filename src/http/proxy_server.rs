use anyhow::Result;
use std::env;
use std::io;
use std::net::SocketAddr;
use std::net::TcpStream;

use console::style;
use fastcgi_client::Client;
use fastcgi_client::Params;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use std::convert::Infallible;
use hyper::header::HeaderValue;

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
    println!("{} {}", req.method(), req.uri());

    let remote_addr = remote_addr.ip().to_string();
    let remote_addr = remote_addr.as_str();

    let script_filename = env::current_dir().unwrap().join("index.php");

    let script_filename = script_filename.to_str().unwrap();
    let script_name = req.uri().to_string();

    let stream = TcpStream::connect(("127.0.0.1", php_port)).unwrap();
    let mut client = Client::new(stream, false);

    let http_port_str = http_port.to_string();
    let php_port_str = php_port.to_string();

    let headers = req.headers();

    let empty_header = &HeaderValue::from_str("").unwrap();

    // Fastcgi params, please reference to nginx-php-fpm config.
    let params = Params::with_predefine()
        .set_request_method("GET")
        .set_script_name(&script_name)
        .set_script_filename(script_filename)
        .set_request_uri(&script_name)
        .set_document_uri(&script_name)
        .set_remote_addr(remote_addr)
        .set_remote_port(http_port_str.as_ref())
        .set_server_addr("127.0.0.1")
        .set_server_port(php_port_str.as_ref())
        .set_server_name("Rymfony")
        .set_content_type(headers.get("Content-Type").unwrap_or(empty_header).to_str().unwrap_or(""))
        .set_content_length(headers.get("Content-Length").unwrap_or(empty_header).to_str().unwrap_or(""));

    let output = client.do_request(&params, &mut io::empty()).unwrap();

    let stdout = output.get_stdout();
    let stdout = stdout.unwrap();
    let stdout = String::from_utf8(stdout);

    let resp = Response::builder();

    let stdout = stdout.unwrap();

    println!("Response body: {}", stdout.clone());

    let resp = resp.body(Body::from(stdout.clone())).unwrap();

    Ok(resp)
}

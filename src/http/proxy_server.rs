use std::net::SocketAddr;

use console::style;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Server;
use std::convert::Infallible;
use crate::http::fastcgi_handler::handle;

#[tokio::main]
pub(crate) async fn start<'a>(
    http_port: u16,
    php_port: u16,
    document_root: &'a String,
    script_filename: &'a String
) {
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], http_port));

    // let document_root = document_root.clone();
    // let script_filename = script_filename.clone();

    let make_service = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        let document_root = document_root.clone();
        let script_filename = script_filename.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle(
                    document_root.clone(),
                    script_filename.clone(),
                    remote_addr.clone(),
                    req,
                    http_port,
                    php_port
                )
            }))
        }
    });

    let http_server = Server::bind(&addr).serve(make_service);

    info!(
        "Server listening to {}",
        style(format!("http://{}", addr)).cyan()
    );

    http_server.await.unwrap();
}

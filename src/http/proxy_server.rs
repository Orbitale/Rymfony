use console::style;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
pub(crate) async fn start(addr: SocketAddr) {
    let service_handler = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(request_handler))
    });

    let http_server = Server::bind(&addr).serve(service_handler);

    println!(
        "Server listening to {}",
        style(format!("http://{}", addr)).cyan()
    );

    http_server
        .await
        .expect("An error occured when starting the server");
}

async fn request_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", _req.method(), _req.uri());

    Ok(Response::new("Hello, World\n".into()))
}

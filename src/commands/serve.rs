use clap::ArgMatches;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use hyper::Response;
use hyper::Request;
use hyper::Body;
use hyper::Server;
use hyper::service::make_service_fn;
use hyper::service::service_fn;

#[tokio::main]
pub async fn serve(listen: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let mut sockets = listen.to_socket_addrs().unwrap();

    let addr = SocketAddr::from(sockets.next().unwrap());

    let service_handler = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(request_handler))
    });

    println!("Serving on http://{}", addr);

    let server = Server::bind(&addr).serve(service_handler);

    server.await?;

    Ok(())
}

async fn request_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", _req.method(), _req.uri());

    Ok(Response::new("Hello, World".into()))
}

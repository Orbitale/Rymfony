use std::convert::Infallible;
use std::net::SocketAddr;

use console::style;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::StatusCode;

#[tokio::main]
pub(crate) async fn start(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let fpm_url = format!("http://127.0.0.1:{}", port);

    let make_service_fn = make_service_fn(|socket: &AddrStream| async move {
        Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
            handle_request(&fpm_url, req).await
        }))
    });

    let http_server = Server::bind(&addr).serve(make_service_fn);

    println!(
        "Server listening to {}",
        style(format!("http://{}", addr)).cyan()
    );

    http_server
        .await
        .expect("An error occured when starting the server");
}

async fn handle_request(fpm_url: &str, req: Request<Body>) -> Response<Body> {
    let method = req.method();

    println!("{} {}", method, req.uri());

    let fpm_url = format!("{}{}", fpm_url, req.uri());

    let mut surf_response = match method.as_str() {
        "GET" => surf::get(fpm_url).await,
        "POST" => surf::post(fpm_url).await,
        "PUT" => surf::put(fpm_url).await,
        "PATCH" => surf::patch(fpm_url).await,
        "HEAD" => surf::head(fpm_url).await,
        "OPTIONS" => surf::options(fpm_url).await,
        "TRACE" => surf::trace(fpm_url).await,
        "CONNECT" => surf::connect(fpm_url).await,
        "DELETE" => surf::delete(fpm_url).await,
        _ => panic!(format!("Unsupported method {}", method)),
    }
    .unwrap();

    // TODO: find a way to make this work too, it's **mandatory**!
    // for header in surf_response.headers().iter() {
    //     hyper_response.header(header.0, header.1);
    // }

    let status = surf_response.body_bytes();
    let status = status.await.unwrap();

    Response::builder()
        .status(StatusCode::from_bytes(status.as_slice()).unwrap())
        .body(Body::from(surf_response.body_bytes().await.unwrap()))
        .unwrap()
}

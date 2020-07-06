use console::style;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
pub(crate) async fn start(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let fpm_url = format!("http://127.0.0.1:{}", port);

    let make_service_fn = make_service_fn(|socket: &AddrStream| async move {
        let service_fn = service_fn(|req: Request<Body>| async move {
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

            let hyper_response = Response::builder();
            hyper_response.status(surf_response.status().as_u16());
            for header in surf_response.headers().iter() {
                hyper_response.header(header.0, header.1);
            }

            let body = surf_response.body_bytes().await;

            let res = hyper_response.body(body);

            Ok::<_, Infallible>(res)
        });

        Ok::<_, Infallible>(service_fn)
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

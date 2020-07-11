use anyhow::Result;
use console::style;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::StatusCode;
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
pub(crate) async fn start(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let make_svc = make_service_fn(|_conn| async move {
        let request_handler = move |req: Request<Body>| handle(req, port);
        Ok::<_, Infallible>(service_fn(request_handler))
    });
    let http_server = Server::bind(&addr).serve(make_svc);

    println!(
        "Server listening to {}",
        style(format!("http://{}", addr)).cyan()
    );

    http_server
        .await
        .expect("An error occured when starting the server");
}

async fn handle_request(fpm_url: String, req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let method = req.method();

    println!("{} {}", method, req.uri());

    let fpm_url = format!("{}{}", fpm_url, req.uri());

    let surf_response = match method.as_str() {
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
    };

    let mut response_from_proxy = surf_response.unwrap();

    let resp = Response::builder();

    // TODO: find a way to make this work too, it's **mandatory**!
    // response_from_proxy.headers().iter().map(move |header| {
    //     let (header_name, value) = header;
    //     resp.header(header_name, value);
    // });

    anyhow::Result::Ok(
        resp.status(
            StatusCode::from_bytes(response_from_proxy.body_bytes().await.unwrap().as_slice())
                .unwrap(),
        )
        .body(Body::from(response_from_proxy.body_bytes().await.unwrap()))
        .unwrap(),
    )
}

async fn handle(req: Request<Body>, port: u16) -> Result<Response<Body>, anyhow::Error> {
    let fpm_url = format!("http://127.0.0.1:{}", port);

    handle_request(fpm_url.clone(), req).await
}

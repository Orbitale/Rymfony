use std::net::SocketAddr;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use console::style;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper_staticfile::Static;

use crate::http::fastcgi_handler::handle_fastcgi;

#[tokio::main]
pub(crate) async fn start<'a>(
    http_port: u16,
    php_port: u16,
    document_root: &'a String,
    script_filename: &'a String
) {
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], http_port));
    let static_files_server = Static::new(Path::new(document_root));

    // let document_root = document_root.clone();
    // let script_filename = script_filename.clone();

    let make_service = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        let document_root = document_root.clone();
        let script_filename = script_filename.clone();
        let static_files_server = static_files_server.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let request_uri = req.uri();
                let request_path = req.uri().path();

                let http_version = crate::http::version::as_str(req.version());

                info!("{} {} {}", http_version, req.method(), request_uri);

                let static_doc_root = Path::new(&document_root);

                let temporary_path: PathBuf;
                let mut render_static = "";
                if static_doc_root.join(request_path).exists() {
                    // Docroot/path
                    temporary_path = static_doc_root.join(request_path);
                    render_static = temporary_path.to_str().unwrap();
                } else if static_doc_root.join("public").join(request_path).exists() {
                    // Docroot/public/path
                    temporary_path = static_doc_root.join("public").join(request_path);
                    render_static = temporary_path.to_str().unwrap();
                } else if static_doc_root.join("web").join(request_path).exists() {
                    // Docroot/web/path
                    temporary_path = static_doc_root.join("web").join(request_path);
                    render_static = temporary_path.to_str().unwrap();
                }
                if render_static != "" {
                    info!("Render static file");
                    return serve_static(req, static_files_server.clone());
                }

                return handle_fastcgi(
                    document_root.clone(),
                    script_filename.clone(),
                    remote_addr.clone(),
                    req,
                    http_port,
                    php_port
                );
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

async fn serve_static(
    req: Request<Body>,
    static_files_server: Static
) -> anyhow::Result<Response<Body>> {
    let static_files_server = static_files_server.clone();
    let response_future = static_files_server.serve(req);

    let response = response_future.await;

    anyhow::Result::Ok(response.unwrap())
}

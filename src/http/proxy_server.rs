use std::convert::Infallible;
use std::net::SocketAddr;
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
    script_filename: &'a String,
) {
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], http_port));
    let static_files_server = Static::new(Path::new(document_root));

    let document_root = document_root.clone();
    let script_filename = script_filename.clone();

    let make_service = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        let document_root = document_root.clone();
        let script_filename = script_filename.clone();
        let static_files_server = static_files_server.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let document_root = document_root.clone();
                let script_filename = script_filename.clone();
                let static_files_server = static_files_server.clone();
                async move {
                    let request_uri = req.uri();
                    let request_path = request_uri.path();

                    let http_version = crate::http::version::as_str(req.version());

                    let render_static = get_render_static_path(&document_root, &request_path);
                    let render_static = !request_path.contains(".php")
                        && render_static != ""
                        && request_path != ""
                        && request_path != "/";

                    info!(
                        "{} {} {}{}",
                        http_version,
                        req.method(),
                        request_uri,
                        if render_static { " (static)" } else { "" }
                    );

                    if render_static {
                        return serve_static(req, static_files_server.clone()).await;
                    }

                    return handle_fastcgi(
                        document_root.clone(),
                        script_filename.clone(),
                        remote_addr.clone(),
                        req,
                        http_port,
                        php_port,
                    )
                    .await;
                }
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

async fn serve_static(
    req: Request<Body>,
    static_files_server: Static,
) -> anyhow::Result<Response<Body>> {
    let static_files_server = static_files_server.clone();
    let response_future = static_files_server.serve(req);

    let response = response_future.await;

    anyhow::Result::Ok(response.unwrap())
}

fn get_render_static_path(document_root: &str, request_path: &str) -> String {
    let directory_separators: &[_] = &['/', '\\'];
    let request_path = request_path.trim_start_matches(directory_separators);
    let document_root = document_root.trim_end_matches(directory_separators);

    let static_doc_root = PathBuf::from(&document_root);

    let docroot_path = PathBuf::from(&static_doc_root).join(request_path);

    let docroot_public_path = PathBuf::from(&static_doc_root)
        .join("public")
        .join(request_path);

    let docroot_web_path = PathBuf::from(&static_doc_root)
        .join("web")
        .join(request_path);

    let mut render_static: &str = "";

    if docroot_path.is_dir() {
        render_static = docroot_path.to_str().unwrap();
    } else if docroot_public_path.is_dir() {
        render_static = docroot_public_path.to_str().unwrap();
    } else if docroot_web_path.is_dir() {
        render_static = docroot_web_path.to_str().unwrap();
    }

    String::from(render_static)
}

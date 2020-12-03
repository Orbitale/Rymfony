use crate::http::fastcgi_handler::handle_fastcgi;

use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

use console::style;
use hyper::Body;
use hyper::Request;
use hyper_staticfile::Static;

use warp::Filter;
use warp::method;
use warp::http::Response;
use http::Method;
use http::HeaderMap;
use warp::filters::path::FullPath;
use warp::filters::header::headers_cloned;
use std::collections::HashMap;
use hyper::body::Bytes;
use std::convert::Infallible;

use crate::config::certificates::get_cert_path;

#[tokio::main]
pub(crate) async fn start(
    use_tls: bool,
    forward_http_to_https: bool,
    http_port: u16,
    php_port: u16,
    document_root: String,
    php_entrypoint_file: String,
) {
    let http_port = http_port.clone();
    let php_port = php_port.clone();
    let document_root = document_root.clone();
    let php_entrypoint_file = php_entrypoint_file.clone();

    let routes = warp::any()
        .and(warp::addr::remote())
        .and(method())
        .and(warp::path::full())
        .and(warp::query::<HashMap<String, String>>())
        .and(headers_cloned())
        .and(warp::body::bytes())
        .and_then(move |
            remote_addr: Option<SocketAddr>,
            method: Method,
            request_path: FullPath,
            query: HashMap<String, String>,
            headers: HeaderMap,
            body: Bytes
        | {
            let http_port = http_port.clone();
            let php_port = php_port.clone();
            let document_root = document_root.clone();
            let php_entrypoint_file = php_entrypoint_file.clone();
            let method = method.clone();

            async move {
                let query_string: String = query.iter()
                    .map(|(key, value)| {
                        format!("{}={}", key, value)
                    })
                    .collect::<Vec<String>>()
                    .join("&")
                    ;

                let request_path = request_path.as_str();
                let mut request_uri = request_path.to_string();

                if query_string.len() > 0 {
                    request_uri.push_str("?");
                    request_uri.push_str(&query_string);
                }

                let mut req = http::Request::builder()
                    .method(&method)
                    .uri(&request_uri)
                    .body(Body::from(body))
                    .unwrap();
                { *req.headers_mut() = headers; }

                let render_static = get_render_static_path(&document_root, &request_path);
                let render_static = !request_path.contains(".php")
                    && render_static != ""
                    && request_path != ""
                    && request_path != "/";

                info!(
                    "{} {}{}",
                    style(method.as_str()).yellow(),
                    style(&request_uri).cyan(),
                    if render_static { " (static)" } else { "" }
                );

                let mut response =
                    if render_static {
                        serve_static(req, Static::new(Path::new(&document_root))).await
                    } else {
                        trace!("Forwarding to FastCGI");

                        let remote_addr = remote_addr.unwrap();

                        handle_fastcgi(
                            &document_root,
                            &php_entrypoint_file,
                            remote_addr,
                            req,
                            &http_port,
                            &php_port,
                            use_tls
                        )
                            .await
                    }
                    ;
                response.as_mut().unwrap().headers_mut().append("server", "rymfony".parse().unwrap());

                response
            }
        })
    ;

    if use_tls {
        let (cert_path, key_path) = get_cert_path()
            .expect("Could not generate TLS certificate");

        warp::serve(routes)
            .tls()
            .cert_path(cert_path)
            .key_path(key_path)
            .run(([127, 0, 0, 1], http_port)).await

    } else {
        warp::serve(routes)
            .run(([127, 0, 0, 1], http_port)).await
    };
}

async fn serve_static(
    req: Request<Body>,
    static_files_server: Static,
) -> Result<Response<Body>, Infallible> {
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

    if docroot_path.is_file() {
        render_static = docroot_path.to_str().unwrap();
        debug!("Static file {} found in document root.", &render_static);
    } else if docroot_public_path.is_file() {
        render_static = docroot_public_path.to_str().unwrap();
        debug!("Static file {} found in \"public/\" subdirectory.", &render_static);
    } else if docroot_web_path.is_file() {
        debug!("Static file {} found in \"web/\" subdirectory.", &render_static);
        render_static = docroot_web_path.to_str().unwrap();
    } else {
        debug!("No static file found based on \"{}\" path.", request_path);
    }

    String::from(render_static)
}

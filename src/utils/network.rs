use std::{net::TcpListener};

pub(crate) fn find_port(default_port: u16) -> Option<u16> {
    for port in default_port..65535 {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_p) => return Some(port),
            _ => {}
        }
    }

    None
}

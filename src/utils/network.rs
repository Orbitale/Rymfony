use std::net::TcpListener;

pub(crate) fn find_available_port(start_from_port: u16) -> u16 {
    for port in start_from_port..65535 {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_p) => return port,
            _ => {}
        }
    }

    panic!("Unable to detect an available port starting from {}", &start_from_port);
}

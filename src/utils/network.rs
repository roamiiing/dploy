use std::net::TcpListener;

const ERROR_TEXT: &str = "Failed to acquire a free port";

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect(ERROR_TEXT)
        .local_addr()
        .expect(ERROR_TEXT)
        .port()
}

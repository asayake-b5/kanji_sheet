use std::net::TcpListener;

pub mod arg_parsing;
pub mod pages;
pub mod pdf_creation;

pub fn find_free_port() -> Option<u16> {
    (8000..55000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

pub async fn launch_browser(url: &str) {
    if webbrowser::open(url).is_ok() {}
}

use std::{net::TcpStream, process::exit};

use app::AppState;
use iced::{Task, Theme};

mod app;

#[tokio::main]
async fn main() {
    let tcp_stream = TcpStream::connect("localhost:8000").unwrap();
    let app_state = app::AppState::new(tcp_stream.try_clone().unwrap());
    iced::application("LetsChat", app::update, app::view)
        .theme(|_m| Theme::KanagawaLotus)
        .subscription(app::subscription)
        .run_with(|| (app_state, Task::none()))
        .unwrap();
    tcp_stream.shutdown(std::net::Shutdown::Both).unwrap();
    exit(0);
}

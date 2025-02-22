use std::process::exit;

use iced::Theme;

mod app;

#[tokio::main]
async fn main() {
    iced::application("LetsChat", app::update, app::view)
        .theme(|_m| Theme::KanagawaLotus)
        .run()
        .unwrap();
    exit(0);
}

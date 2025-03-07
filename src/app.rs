mod chat;
mod welcome;
pub mod security;

use std::net::TcpStream;

use iced::{Element, Subscription};

enum Screen {
    WelcomeScreen(welcome::WelcomeViewState),
    ChatScreen(chat::ChatViewState),
}

pub struct AppState {
    screen: Screen,
    tcp_stream: TcpStream,
}

impl AppState {
    pub fn new(tcp_stream: TcpStream) -> Self {
        AppState {
            screen: Screen::WelcomeScreen(welcome::WelcomeViewState::new(
                tcp_stream.try_clone().unwrap(),
            )),
            tcp_stream,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    WelcomeMessages(welcome::WelcomeViewMessage),
    ChatMessages(chat::ChatViewMessage),
}

pub fn update(app_state: &mut AppState, message: AppMessage) {
    match message {
        AppMessage::WelcomeMessages(welcome_view_message) => {
            if let Screen::WelcomeScreen(welcome_view_state) = &mut app_state.screen {
                let action = welcome::welcome_view_update(welcome_view_state, welcome_view_message);
                match action {
                    welcome::WelcomeViewAction::RoomJoined(success_message, username, room_id) => {
                        app_state.screen = Screen::ChatScreen(chat::ChatViewState::new(
                            vec![success_message],
                            username,
                            room_id,
                            app_state.tcp_stream.try_clone().unwrap(),
                        ));
                    }
                    welcome::WelcomeViewAction::None => {}
                }
            }
        }
        AppMessage::ChatMessages(chat_view_message) => {
            if let Screen::ChatScreen(chat_view_state) = &mut app_state.screen {
                let action = chat::update(chat_view_state, chat_view_message);
                match action {
                    chat::ChatViewAction::None => {}
                    chat::ChatViewAction::Disconnect => {
                        // restart app
                    }
                }
            }
        }
    }
}

pub fn view(app_state: &AppState) -> Element<AppMessage> {
    match &app_state.screen {
        Screen::ChatScreen(m) => chat::view(m).map(AppMessage::ChatMessages),
        Screen::WelcomeScreen(m) => welcome::welcome_view(m).map(AppMessage::WelcomeMessages),
    }
}

pub fn subscription(app_state: &AppState) -> Subscription<AppMessage> {
    if let Screen::ChatScreen(_m) = &app_state.screen {
        return Subscription::run(chat::recv_updates).map(AppMessage::ChatMessages);
    }
    Subscription::none()
}

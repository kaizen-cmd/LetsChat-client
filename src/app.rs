mod chat;
mod welcome;

use std::{
    net::TcpStream,
    sync::{LazyLock, Mutex},
};

use iced::{futures::channel::mpsc::Sender, Element};

static TCP_STREAM: LazyLock<Mutex<TcpStream>> =
    LazyLock::new(|| Mutex::new(TcpStream::connect("localhost:8000").unwrap()));

enum Screen {
    WelcomeScreen(welcome::WelcomeViewState),
    ChatScreen(chat::ChatViewState),
}

pub struct AppState {
    screen: Screen,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            screen: Screen::WelcomeScreen(welcome::WelcomeViewState::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AbstractMessage {
    WelcomeMessages(welcome::WelcomeViewMessage),
    ChatMessages(chat::ChatViewMessage),
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    WelcomeMessages(welcome::WelcomeViewMessage),
    ChatMessages(chat::ChatViewMessage),
    SubscriptionMessage(Sender<String>, AbstractMessage),
}

pub fn update(app_state: &mut AppState, message: AppMessage) {
    match message {
        AppMessage::WelcomeMessages(welcome_view_message) => {
            if let Screen::WelcomeScreen(welcome_view_state) = &mut app_state.screen {
                let action = welcome::welcome_view_update(welcome_view_state, welcome_view_message);
                match action {
                    welcome::WelcomeViewAction::RoomJoined => {
                        app_state.screen = Screen::ChatScreen(chat::ChatViewState::default());
                    }
                    welcome::WelcomeViewAction::None => {}
                }
            }
        }
        AppMessage::ChatMessages(m) => {}
        AppMessage::SubscriptionMessage(sx, am) => {}
    }
}

pub fn view(app_state: &AppState) -> Element<AppMessage> {
    match &app_state.screen {
        Screen::ChatScreen(m) => chat::view(m).map(AppMessage::ChatMessages),
        Screen::WelcomeScreen(m) => welcome::welcome_view(m).map(AppMessage::WelcomeMessages),
    }
}

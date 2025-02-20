mod chat_screen;
mod welcome_screen;

use chat_screen::chat;
use iced::Element;
use welcome_screen::welcome;

enum Screen {
    WelcomeScreen(welcome::Client),
    ChatScreen(chat::Client),
}

struct ScreenState {
    screen: Screen,
}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            screen: Screen::WelcomeScreen(welcome::Client::default()),
        }
    }
}

#[derive(Clone, Debug)]
enum EventMessage {
    WelcomeMessage(welcome::Message),
    ChatMessage(chat::Message),
}

fn update(screen_state: &mut ScreenState, message: EventMessage) {
    match message {
        EventMessage::WelcomeMessage(m) => {
            if let Screen::WelcomeScreen(s) = &mut screen_state.screen {
                let action = s.update(m);
                match action {
                    welcome::Action::None => {}
                    welcome::Action::Connected => {
                        screen_state.screen = Screen::ChatScreen(chat::Client::default());
                    }
                }
            }
        }
        EventMessage::ChatMessage(m) => {
            if let Screen::ChatScreen(s) = &mut screen_state.screen {
                s.update(m);
            }
        }
    }
}

fn view(screen_state: &ScreenState) -> Element<EventMessage> {
    match &screen_state.screen {
        Screen::ChatScreen(client) => client.view().map(EventMessage::ChatMessage),
        Screen::WelcomeScreen(client) => client.view().map(EventMessage::WelcomeMessage),
    }
}

fn main() {
    let _ = iced::run("Chat Client", update, view).unwrap();
}

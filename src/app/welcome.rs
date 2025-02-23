use crate::app::TCP_STREAM;

use std::io::{Read, Write};

use iced::widget::{button, column, text, text_input};
use iced::Element;

pub struct WelcomeViewState {
    welcome_message: String,
    room_id_text: String,
    name_text: String,
}

impl WelcomeViewState {
    pub fn new() -> Self {
        let mut welcome_view_state = WelcomeViewState {
            welcome_message: String::from("Connecting.."),
            room_id_text: String::new(),
            name_text: String::new(),
        };

        let tcp_stream_locked = TCP_STREAM.lock().unwrap();
        let mut tcp_stream = tcp_stream_locked.try_clone().unwrap();
        drop(tcp_stream_locked);
        let mut buf = [0u8; 1024];
        let bytes_read = tcp_stream.read(&mut buf).unwrap();
        let message = std::str::from_utf8(&buf[..bytes_read]).unwrap();
        welcome_view_state.welcome_message = message.to_string();

        welcome_view_state
    }
}

#[derive(Clone, Debug)]
pub enum WelcomeViewMessage {
    NameChanged(String),
    RoomIdChanged(String),
    SbmitForm,
}

#[derive(Debug, Clone)]
pub enum WelcomeViewAction {
    // success_message, name, room_id
    RoomJoined(String, String, String),
    None,
}

pub fn welcome_view_update(
    welcome_view_state: &mut WelcomeViewState,
    message: WelcomeViewMessage,
) -> WelcomeViewAction {
    match message {
        WelcomeViewMessage::NameChanged(s) => {
            welcome_view_state.name_text = s;
            WelcomeViewAction::None
        }
        WelcomeViewMessage::RoomIdChanged(s) => {
            welcome_view_state.room_id_text = s;
            WelcomeViewAction::None
        }
        WelcomeViewMessage::SbmitForm => {
            let tcp_stream_locked = TCP_STREAM.lock().unwrap();
            let mut tcp_stream = tcp_stream_locked.try_clone().unwrap();
            drop(tcp_stream_locked);

            let message = format!(
                "{} {}",
                welcome_view_state.room_id_text, welcome_view_state.name_text
            );
            tcp_stream.write_all(message.as_bytes()).unwrap();

            let mut buf = [0u8; 1024];
            let bytes_read = tcp_stream.read(&mut buf).unwrap();
            let message = std::str::from_utf8(&buf[..bytes_read]).unwrap();
            if message.contains("Room ID") {
                return WelcomeViewAction::RoomJoined(
                    message.to_string(),
                    welcome_view_state.name_text.to_string(),
                    welcome_view_state.room_id_text.to_string(),
                );
            }
            welcome_view_state.welcome_message.push('\n');
            welcome_view_state.welcome_message.push_str(message);
            WelcomeViewAction::None
        }
    }
}

pub fn welcome_view(welcome_view_state: &WelcomeViewState) -> Element<WelcomeViewMessage> {
    let welcome_text: Element<WelcomeViewMessage> =
        text(&welcome_view_state.welcome_message).into();
    let name_ip: Element<WelcomeViewMessage> =
        text_input("What is your name?", &welcome_view_state.name_text)
            .on_input(WelcomeViewMessage::NameChanged)
            .into();
    let room_id_ip: Element<WelcomeViewMessage> = text_input(
        "Which room do you want to join?",
        &welcome_view_state.room_id_text,
    )
    .on_input(WelcomeViewMessage::RoomIdChanged)
    .into();
    let connect_btn: Element<WelcomeViewMessage> = button("Connect")
        .on_press(WelcomeViewMessage::SbmitForm)
        .into();
    column![welcome_text, name_ip, room_id_ip, connect_btn].into()
}

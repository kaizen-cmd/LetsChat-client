use crate::app::TCP_STREAM;

use std::io::{Read, Write};

use iced::futures::channel::mpsc::Sender;
use iced::futures::stream::FusedStream;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, column, text, text_input};
use iced::{stream, Element};

pub struct WelcomeViewState {
    welcome_message: String,
    room_id_text: String,
    name_text: String,
}

impl Default for WelcomeViewState {
    fn default() -> Self {
        WelcomeViewState {
            welcome_message: String::from("Connecting.."),
            room_id_text: String::new(),
            name_text: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum WelcomeViewMessage {
    ReadMessage(Sender<String>),
    WelcomeMessageChanged(String),
    NameChanged(String),
    RoomIdChanged(String),
    SbmitForm,
}

#[derive(Debug, Clone)]
pub enum WelcomeViewAction {
    RoomJoined(String),
    None,
}

pub fn welcome_view_update(
    welcome_view_state: &mut WelcomeViewState,
    message: WelcomeViewMessage,
) -> WelcomeViewAction {
    match message {
        WelcomeViewMessage::ReadMessage(mut sx) => {
            println!("Message::StartReader received");
            let tcp_stream_locked = TCP_STREAM.lock().unwrap();
            let mut tcp_stream = tcp_stream_locked.try_clone().unwrap();
            drop(tcp_stream_locked);
            tokio::spawn(async move {
                println!("Reader Thread started");
                let mut buf = [0u8; 1024];
                let bytes_read = tcp_stream.read(&mut buf).unwrap();
                let message = std::str::from_utf8(&buf[..bytes_read]).unwrap();
                println!("Reader Thread Message Received");
                sx.send(message.to_string()).await.unwrap();
                println!("Reader Thread Message sent to the mpsc channel {}", message);
            });
            WelcomeViewAction::None
        }
        WelcomeViewMessage::WelcomeMessageChanged(s) => {
            welcome_view_state.welcome_message = s;
            WelcomeViewAction::None
        }
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
                return WelcomeViewAction::RoomJoined(message.to_string());
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

pub fn recv_updates() -> impl Stream<Item = WelcomeViewMessage> {
    println!("Subscription running");
    stream::channel(100, |mut op| async move {
        println!("Started stream channel insided subscription");
        let (sx, mut rx) = iced::futures::channel::mpsc::channel(100);
        println!("Created mpsc channel");
        op.send(WelcomeViewMessage::ReadMessage(sx.clone())).await.unwrap();
        println!("Sent sender by Message::StartReader");
        let message = rx.select_next_some().await;
        op.send(WelcomeViewMessage::WelcomeMessageChanged(message)).await.unwrap();
    })
}
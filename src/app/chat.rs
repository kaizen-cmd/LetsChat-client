use crate::app::TCP_STREAM;

use core::str;
use std::{
    io::{Read, Write},
    sync::Arc,
};

use iced::{
    futures::{SinkExt, StreamExt},
    widget::{button, column, container, scrollable, text, text_input, Column, Row},
    Color, Element, Length,
};

use std::sync::Mutex;

pub struct ChatViewState {
    name: String,
    room_id: String,
    messages: Arc<Mutex<Vec<String>>>,
    current_message: String,
}

impl Default for ChatViewState {
    fn default() -> Self {
        ChatViewState {
            name: String::new(),
            room_id: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
            current_message: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatViewMessage {
    StartReader(iced::futures::channel::mpsc::Sender<String>),
    ReceivedMessage(String),
    SendMessage(String),
    CurrentMessageChanged(String),
}


fn update(app_state: &mut ChatViewState, message: ChatViewMessage) {
    match message {
        ChatViewMessage::StartReader(mut sx) => {
            println!("Message::StartReader received");
            let tcp_stream_locked = TCP_STREAM.lock().unwrap();
            let mut tcp_stream = tcp_stream_locked.try_clone().unwrap();
            drop(tcp_stream_locked);
            tokio::spawn(async move {
                println!("Reader Thread started");
                loop {
                    let mut buf = [0u8; 1024];
                    let bytes_read = tcp_stream.read(&mut buf).unwrap();
                    let message = str::from_utf8(&buf[..bytes_read]).unwrap();
                    println!("Reader Thread Message Received");
                    sx.send(message.to_string()).await.unwrap();
                    println!("Reader Thread Message sent to the mpsc channel");
                }
            });
        }
        ChatViewMessage::ReceivedMessage(s) => {
            let mut messages = app_state.messages.lock().unwrap();
            messages.push(s);
            drop(messages);
        }
        ChatViewMessage::SendMessage(s) => {
            let tcp_stream_locked = TCP_STREAM.lock().unwrap();
            let mut tcp_stream = tcp_stream_locked.try_clone().unwrap();
            drop(tcp_stream_locked);
            let s = s.trim().to_string();
            tcp_stream.write_all(s.as_bytes()).unwrap();
            {
                app_state
                    .messages
                    .lock()
                    .unwrap()
                    .push(format!("You > {}", s));
            }
            app_state.current_message.clear();
        }
        ChatViewMessage::CurrentMessageChanged(s) => {
            app_state.current_message = s;
        }
    }
}

pub fn view(app_state: &ChatViewState) -> Element<ChatViewMessage> {
    let messages = app_state.messages.lock().unwrap();

    let messages_text_vec = messages
        .iter()
        .map(|msg| {
            if msg.starts_with("You > ") {
                text(msg.clone()).size(15).color(Color::BLACK).into()
            } else {
                text(msg.clone())
                    .color(Color::from_rgb(255.0, 0.0, 0.0))
                    .size(15)
                    .into()
            }
        })
        .collect::<Vec<Element<ChatViewMessage>>>();

    drop(messages);

    let messages_column: Element<ChatViewMessage> = Column::from_vec(messages_text_vec).into();
    let scrollable_messages: Element<ChatViewMessage> = scrollable(messages_column)
        .height(Length::Fill)
        .width(Length::Fill)
        .into();

    let message_input: Element<ChatViewMessage> =
        text_input("Type your message", &app_state.current_message)
            .on_input(ChatViewMessage::CurrentMessageChanged)
            .on_submit(ChatViewMessage::SendMessage(
                app_state.current_message.clone(),
            ))
            .padding(10)
            .size(16)
            .into();

    let send_button: Element<ChatViewMessage> = button("Send")
        .on_press(ChatViewMessage::SendMessage(
            app_state.current_message.clone(),
        ))
        .padding(10)
        .into();

    let input_row: Element<ChatViewMessage> = Row::new()
        .push(message_input)
        .push(send_button)
        .spacing(10)
        .height(Length::Shrink)
        .into();

    column![
        scrollable_messages,
        container(input_row).padding(10).width(Length::Fill)
    ]
    .spacing(10)
    .height(Length::Fill)
    .padding(10)
    .into()
}


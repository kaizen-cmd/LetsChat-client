use core::str;
use std::{
    io::{Read, Write},
    net::TcpStream,
    process::exit,
    sync::Arc,
};

use iced::{
    font::Weight, futures::{SinkExt, Stream, StreamExt}, stream, widget::{button, column, container, scrollable, text, text_input, Column, Row}, Color, Element, Font, Length, Subscription, Theme
};

use std::sync::{LazyLock, Mutex};

static TCP_STREAM: LazyLock<Mutex<TcpStream>> =
    LazyLock::new(|| Mutex::new(TcpStream::connect("localhost:8000").unwrap()));

struct AppState {
    name: String,
    room_id: String,
    messages: Arc<Mutex<Vec<String>>>,
    current_message: String,
    room_joined: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            name: String::new(),
            room_id: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
            current_message: String::new(),
            room_joined: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    StartReader(iced::futures::channel::mpsc::Sender<String>),
    ReceivedMessage(String),
    SendMessage(String),
    CurrentMessageChanged(String),
}

fn subscription(_app_state: &AppState) -> Subscription<Message> {
    Subscription::run(recv_updates)
}

fn recv_updates() -> impl Stream<Item = Message> {
    println!("Subscription running");
    stream::channel(100, |mut op| async move {
        println!("Started stream channel insided subscription");
        let (sx, mut rx) = iced::futures::channel::mpsc::channel(100);
        println!("Created mpsc channel");
        op.send(Message::StartReader(sx)).await.unwrap();
        println!("Sent sender by Message::StartReader");

        loop {
            let message = rx.select_next_some().await;
            op.send(Message::ReceivedMessage(message)).await.unwrap();
        }
    })
}

fn update(app_state: &mut AppState, message: Message) {
    match message {
        Message::StartReader(mut sx) => {
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
        Message::ReceivedMessage(s) => {
            let mut messages = app_state.messages.lock().unwrap();
            messages.push(s);
            drop(messages);
        }
        Message::SendMessage(s) => {
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
        Message::CurrentMessageChanged(s) => {
            app_state.current_message = s;
        }
    }
}

fn view(app_state: &AppState) -> Element<Message> {
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
        .collect::<Vec<Element<Message>>>();

    drop(messages);

    let messages_column: Element<Message> = Column::from_vec(messages_text_vec).into();
    let scrollable_messages: Element<Message> = scrollable(messages_column)
        .height(Length::Fill)
        .width(Length::Fill)
        .into();

    let message_input: Element<Message> =
        text_input("Type your message", &app_state.current_message)
            .on_input(Message::CurrentMessageChanged)
            .on_submit(Message::SendMessage(app_state.current_message.clone()))
            .padding(10)
            .size(16)
            .into();

    let send_button: Element<Message> = button("Send")
        .on_press(Message::SendMessage(app_state.current_message.clone()))
        .padding(10)
        .into();

    let input_row: Element<Message> = Row::new()
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

#[tokio::main]
async fn main() {
    let _ = iced::application("Lets Chat", update, view)
        .subscription(subscription)
        .theme(|app_state: &AppState| Theme::GruvboxLight)
        .run()
        .unwrap();
    exit(0);
}

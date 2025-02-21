use core::str;
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};

use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream,
    widget::{column, text, text_input, Column},
    Element, Subscription,
};

use std::sync::{LazyLock, Mutex};

static TCP_STREAM: LazyLock<Mutex<TcpStream>> =
    LazyLock::new(|| Mutex::new(TcpStream::connect("localhost:8000").unwrap()));

struct AppState {
    name: String,
    room_id: String,
    messages: Arc<Mutex<Vec<String>>>,
    current_message: String,
    room_joined: bool
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            name: String::from("Tejas"),
            room_id: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
            current_message: String::new(),
            room_joined: false
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
                loop {
                    println!("Reader Thread started");
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
            app_state.current_message.clear();
        }
        Message::CurrentMessageChanged(s) => {
            app_state.current_message = s;
        }
    }
}

fn view(app_state: &AppState) -> Element<Message> {
    let mut message_text_vec: Vec<Element<Message>> = Vec::new();
    let messages = app_state.messages.lock().unwrap();

    for message in messages.iter() {
        message_text_vec.push(text(message.clone()).into());
    }

    drop(messages);

    let messages_column: Element<Message> = Column::from_vec(message_text_vec).into();

    let message_input: Element<Message> = text_input("Type your message", &app_state.current_message)
        .on_input(Message::CurrentMessageChanged)
        .on_submit(Message::SendMessage(app_state.current_message.clone()))
        .into();

    column![messages_column, message_input].into()
}

#[tokio::main]
async fn main() {
    let _ = iced::application("MyApp", update, view)
        .subscription(subscription)
        .run()
        .unwrap();
}

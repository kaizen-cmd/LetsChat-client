use core::str;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use iced::widget::{column, text, text_input};
use iced::{Element, Subscription};

struct Client {
    tcp_stream: TcpStream,
    name: String,
    room_id: String,
    messages: Arc<Mutex<Vec<String>>>,
    current_message: String,
    receiver: Receiver<String>
}

impl Client {
    fn create() -> Self {

        let (sx, rx) = std::sync::mpsc::channel();
        let tcp_stream = TcpStream::connect("localhost:8000").expect("Failed to connect to server");

        let mut tcp_stream_clone = tcp_stream.try_clone().unwrap();
        let sender_clone = sx.clone();

        thread::spawn(move || {
            loop {
                let mut buf = [0u8; 1024];
                let bytes_read = tcp_stream_clone.read(&mut buf).unwrap();
                if bytes_read == 0 {
                    break;
                }
                let message = str::from_utf8(&buf[..bytes_read]).unwrap();
                sender_clone.send(message.to_string()).unwrap();
            }
        }); 

        let client = Client {
            tcp_stream: tcp_stream.try_clone().unwrap(),
            name: String::new(),
            room_id: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
            current_message: String::new(),
            receiver: rx
        };

        client
    }

    // fn subscription(&self) -> Subscription<Message> {

    // }


}


impl Default for Client {
    fn default() -> Self {
        Self::create()
    }
}

#[derive(Debug, Clone)]
enum Message {
    CurrentMessageChanged(String),
    SendMessage(String),
    UpdateMessages(String),
}

fn update(client: &mut Client, message: Message) {
    match message {
        Message::CurrentMessageChanged(msg) => {
            client.current_message = msg;
        }
        Message::SendMessage(msg) => {
            client
                .tcp_stream
                .write_all(msg.trim().to_string().as_bytes())
                .expect("Failed to write to stream");
            client.current_message.clear();
        }
        Message::UpdateMessages(msg) => {
            let mut messages = client.messages.lock().unwrap();
            messages.push(msg);
            drop(messages);
        }
    }
}


fn view(client: &Client) -> Element<Message> {
    let messages = client.messages.lock().unwrap();
    let messages_string = messages.join("\n").to_string();
    drop(messages);
    let text = text(messages_string).size(20);
    let text_input = text_input("Message", &client.current_message)
        .on_input(Message::CurrentMessageChanged)
        .on_submit(Message::SendMessage(client.current_message.clone()));
    column![text, text_input].into()
}

fn main() {
    let _ = iced::application("ChatClient", update, view);
}

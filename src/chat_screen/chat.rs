use core::str;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use iced::{
    widget::{button, column, container, text, text_input},
    Element,
};

#[derive(Debug, Clone)]
pub enum Message {
    RoomIdIpValueChanged(String),
    UsernameIpValueChange(String),
    SubmitInfoForm,
}

pub struct Client {
    room_id: String,
    username: String,
    welcome_message: String,
    tcp_stream: TcpStream,
}

impl Default for Client {
    fn default() -> Self {
        let mut buf = [0u8; 1024];
        let mut tcp_stream = TcpStream::connect("localhost:8000").unwrap();
        let bytes_read = tcp_stream.read(&mut buf).unwrap();
        let message = str::from_utf8(&buf[..bytes_read]).unwrap();

        Client {
            room_id: String::new(),
            username: String::new(),
            welcome_message: message.to_string(),
            tcp_stream,
        }
    }
}

impl Client {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::RoomIdIpValueChanged(s) => {
                self.room_id = s;
            }
            Message::UsernameIpValueChange(s) => {
                self.username = s;
            }
            Message::SubmitInfoForm => {
                let mut message = self.room_id.clone();
                message.push(' ');
                message.push_str(&self.username);
                self.tcp_stream.write(message.as_bytes()).unwrap();

                let mut buf = [0u8; 1024];
                let bytes_read = self.tcp_stream.read(&mut buf).unwrap();
                self.welcome_message = str::from_utf8(&buf[..bytes_read]).unwrap().to_string();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let welcome_message_text = text(&self.welcome_message);
        let username_ip = text_input("What is your name?", &self.username)
            .on_input(Message::UsernameIpValueChange);
        let room_id_ip = text_input("Which room do you want to join?", &self.room_id)
            .on_input(Message::RoomIdIpValueChanged);
        let submit_btn = button("Chat Screen").on_press(Message::SubmitInfoForm);
        let info_form_column = column![welcome_message_text, username_ip, room_id_ip, submit_btn];
        let container = container(info_form_column).padding(10);
        container.into()
    }
}

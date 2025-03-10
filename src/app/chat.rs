use core::str;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};

use base64::Engine;
use iced::{
    advanced::graphics::core::font,
    futures::{stream::FusedStream, SinkExt, Stream, StreamExt},
    stream,
    widget::{button, column, container, row, scrollable, text, text_input, Column, Row},
    Alignment, Color, Element, Font, Length, Padding,
};
use std::sync::Mutex;

use super::security::{decrypt, encrypt};

pub struct ChatViewState {
    name: String,
    room_id: String,
    messages: Arc<Mutex<Vec<ConversationMessage>>>,
    current_message: String,
    tcp_stream: TcpStream,
    conversation_message_manager: ConversationMessageManager,
}

impl ChatViewState {
    pub fn new(
        mut messages: Vec<String>,
        name: String,
        room_id: String,
        tcp_stream: TcpStream,
    ) -> Self {
        messages.push(format!("ROOM_JOIN_SUCCESS_MESSAGE You have joined as {}", name).to_string());
        let mut cmm = ConversationMessageManager::new();
        let messages = cmm.cms_from_vec(messages);
        ChatViewState {
            name,
            room_id,
            messages: Arc::new(Mutex::new(messages)),
            current_message: String::new(),
            tcp_stream,
            conversation_message_manager: cmm,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatViewMessage {
    StartReader(iced::futures::channel::mpsc::Sender<String>),
    ReceivedMessage(String),
    SendMessage(String),
    CurrentMessageChanged(String),
    JoinVoiceChannel,
    Disconnect,
}

pub enum ChatViewAction {
    None,
    Disconnect,
}

pub fn update(app_state: &mut ChatViewState, message: ChatViewMessage) -> ChatViewAction {
    match message {
        ChatViewMessage::StartReader(mut sx) => {
            println!("Message::StartReader received");
            let mut tcp_stream = app_state.tcp_stream.try_clone().unwrap();
            tokio::spawn(async move {
                let key = b"thisIsASecretKey";
                loop {
                    let mut buf = [0u8; 1024];
                    let bytes_read = tcp_stream.read(&mut buf).unwrap();

                    let mut m = str::from_utf8(&buf[..bytes_read]).unwrap().to_string();
                    let split_message = m.split(' ').collect::<Vec<_>>();
                    let type_of_message = split_message[0];

                    if type_of_message == "NORMAL_MESSAGE" {
                        let b64decoded_message = base64::prelude::BASE64_STANDARD
                            .decode(split_message[1])
                            .unwrap();
                        m = format!("{} {}", split_message[0], decrypt(b64decoded_message, key));
                    }

                    match sx.send(m).await {
                        Ok(_) => {}
                        Err(_e) => {
                            break;
                        }
                    };
                }
            });
            ChatViewAction::None
        }
        ChatViewMessage::ReceivedMessage(s) => {
            let mut messages = app_state.messages.lock().unwrap();
            let cm = app_state.conversation_message_manager.cm_from_string(s);
            messages.push(cm);
            drop(messages);
            ChatViewAction::None
        }
        ChatViewMessage::SendMessage(s) => {
            let message = s.clone();
            let s = format!("{} > {}", app_state.name, s.trim().to_string());
            if s.len() == 0 {
                return ChatViewAction::None;
            }
            let mut tcp_stream = app_state.tcp_stream.try_clone().unwrap();

            let key = b"thisIsASecretKey";
            let encrypted_message = encrypt(s.as_str(), key);

            // attach the type of the message
            let encrypted_message_string =
                base64::prelude::BASE64_STANDARD.encode(encrypted_message);
            let final_message = format!("NORMAL_MESSAGE {}", encrypted_message_string);

            tcp_stream.write_all(final_message.as_bytes()).unwrap();
            {
                app_state.messages.lock().unwrap().push(
                    app_state
                        .conversation_message_manager
                        .format_conversation_message("You".to_string(), message),
                );
            }
            app_state.current_message.clear();
            ChatViewAction::None
        }
        ChatViewMessage::CurrentMessageChanged(s) => {
            app_state.current_message = s;
            ChatViewAction::None
        }
        ChatViewMessage::JoinVoiceChannel => {
            let mut tcp_stream = app_state.tcp_stream.try_clone().unwrap();
            tcp_stream.write_all(format!("JOIN_VOICE_CHANNEL_MESSAGE {}", app_state.name).as_bytes()).unwrap();
            ChatViewAction::None
        }
        ChatViewMessage::Disconnect => {
            let tcp_stream = app_state.tcp_stream.try_clone().unwrap();
            match tcp_stream.shutdown(std::net::Shutdown::Both) {
                Ok(_) => {}
                Err(e) => println!("Shutdown failed {}", e),
            };
            ChatViewAction::Disconnect
        }
    }
}

pub fn view(app_state: &ChatViewState) -> Element<ChatViewMessage> {
    let font_size = 17;
    let mut cm_name_font = Font::with_name("clash-grotesk-variable");
    cm_name_font.weight = font::Weight::Bold;
    let mut cm_message = Font::with_name("clash-grotesk-variable");
    cm_message.weight = font::Weight::Normal;

    let messages = app_state.messages.lock().unwrap();
    let messages_text_vec = messages
        .iter()
        .map(|msg| {
            if msg.name.len() != 0 {
                let name_text: Element<ChatViewMessage> = text(format!("{}", msg.name.trim()))
                    .color(msg.color)
                    .size(font_size)
                    .font(cm_name_font)
                    .line_height(0.6)
                    .into();
                let message_text: Element<ChatViewMessage> =
                    text(format!("{}", msg.content.trim()))
                        .color(msg.color)
                        .size(font_size + 1)
                        .into();

                if msg.name == "You" {
                    column![name_text, message_text]
                        .width(Length::Fill)
                        .align_x(Alignment::End)
                        .padding(Padding {
                            top: 0.0,
                            right: 20.0,
                            bottom: 0.0,
                            left: 20.0,
                        })
                        .into()
                } else {
                    column![name_text, message_text]
                        .width(Length::Fill)
                        .padding(Padding {
                            top: 0.0,
                            right: 20.0,
                            bottom: 0.0,
                            left: 20.0,
                        })
                        .into()
                }
            } else {
                let generic_text: Element<ChatViewMessage> = text(format!("{}", msg.content))
                    .color(msg.color)
                    .size(font_size)
                    .into();
                row![generic_text].into()
            }
        })
        .collect::<Vec<Element<ChatViewMessage>>>();

    drop(messages);

    let messages_column: Element<ChatViewMessage> = Column::from_vec(messages_text_vec)
        .width(Length::Fill)
        .spacing(20)
        .into();
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

    let join_voice_btn: Element<ChatViewMessage> = button("Join Voice Chat")
        .on_press(ChatViewMessage::JoinVoiceChannel)
        .into();

    let disconnect_btn: Element<ChatViewMessage> = button("Disconnect Room")
        .on_press(ChatViewMessage::Disconnect)
        .into();

    column![
        join_voice_btn,
        disconnect_btn,
        scrollable_messages,
        container(input_row).padding(10).width(Length::Fill)
    ]
    .spacing(10)
    .height(Length::Fill)
    .padding(20)
    .into()
}

pub fn recv_updates() -> impl Stream<Item = ChatViewMessage> {
    stream::channel(100, |mut op| async move {
        let (sx, mut rx) = iced::futures::channel::mpsc::channel(100);
        op.send(ChatViewMessage::StartReader(sx.clone()))
            .await
            .unwrap();
        loop {
            if sx.is_closed() || rx.is_terminated() {
                break;
            }
            let message = rx.select_next_some().await;
            op.send(ChatViewMessage::ReceivedMessage(message))
                .await
                .unwrap();
        }
    })
}

struct ConversationMessage {
    name: String,
    color: Color,
    content: String,
}

struct ConversationMessageManager {
    colors_list: Vec<Color>,
    current_index: usize,
    color_user_map: HashMap<String, Color>,
}

impl ConversationMessageManager {
    fn new() -> Self {
        ConversationMessageManager {
            colors_list: vec![
                Color::new(0.1, 0.3, 0.6, 1.0), // Soft Blue
                Color::new(0.2, 0.5, 0.3, 1.0), // Muted Green
                Color::new(0.6, 0.3, 0.1, 1.0), // Warm Brown
                Color::new(0.5, 0.2, 0.4, 1.0), // Gentle Purple
                Color::new(0.7, 0.5, 0.2, 1.0), // Soft Mustard
                Color::new(0.2, 0.6, 0.5, 1.0), // Teal Green
                Color::new(0.8, 0.3, 0.3, 1.0), // Soft Red
                Color::new(0.3, 0.7, 0.4, 1.0), // Fresh Green
                Color::new(0.4, 0.4, 0.7, 1.0), // Cool Blue
                Color::new(0.7, 0.4, 0.6, 1.0), // Soft Magenta
                Color::new(0.3, 0.6, 0.7, 1.0), // Aqua Blue
                Color::new(0.6, 0.7, 0.3, 1.0), // Olive Green
                Color::new(0.5, 0.3, 0.7, 1.0), // Lavender
                Color::new(0.2, 0.4, 0.7, 1.0), // Deep Sky Blue
                Color::new(0.7, 0.6, 0.3, 1.0), // Golden Yellow
                Color::new(0.3, 0.7, 0.6, 1.0), // Cyan Green
                Color::new(0.4, 0.3, 0.6, 1.0), // Slate Blue
                Color::new(0.6, 0.4, 0.3, 1.0), // Warm Terracotta
                Color::new(0.2, 0.5, 0.6, 1.0), // Ocean Blue
                Color::new(0.7, 0.3, 0.5, 1.0), // Rosy Pink
            ],
            current_index: 0,
            color_user_map: HashMap::new(),
        }
    }

    fn format_conversation_message(
        &mut self,
        name: String,
        content: String,
    ) -> ConversationMessage {
        let color = match self.color_user_map.get(&name) {
            Some(c) => c.clone(),
            None => {
                self.color_user_map.insert(
                    name.clone(),
                    self.colors_list.get(self.current_index).unwrap().clone(),
                );
                let color = self.colors_list.get(self.current_index).unwrap();
                self.current_index = (self.current_index + 1) % 20;
                color.clone()
            }
        };
        ConversationMessage {
            name,
            content,
            color,
        }
    }

    fn cm_from_string(&mut self, mut msg: String) -> ConversationMessage {
        msg = msg.split(" ").collect::<Vec<_>>()[1..].join(" ");
        if msg.contains(">") {
            let split_msg = msg.split(">").collect::<Vec<&str>>();
            let name = split_msg[0].trim().to_string();
            let message = split_msg[1..].join("").to_string();
            let cm = self.format_conversation_message(name, message);
            cm
        } else {
            ConversationMessage {
                name: String::new(),
                color: Color::BLACK,
                content: msg.clone(),
            }
        }
    }

    fn cms_from_vec(&mut self, v: Vec<String>) -> Vec<ConversationMessage> {
        v.iter()
            .map(|msg| self.cm_from_string(msg.clone()))
            .collect::<Vec<ConversationMessage>>()
    }
}

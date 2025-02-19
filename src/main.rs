use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

// use iced::widget::{column, Button, TextInput};
// use iced::Element;

// #[derive(Debug, Clone)]
// enum Message {
//     NameChanged(String),
//     RoomIdChanged(String),
//     SubmitCredsFrom,
// }

// struct Client {
//     name: String,
//     room_id: String,
// }

// impl Default for Client {
//     fn default() -> Self {
//         Self {
//             name: "Anonymous".to_string(),
//             room_id: "1".to_string(),
//         }
//     }
// }

// impl Client {

//     fn update(&mut self, message: Message) {
//         match message {
//             Message::NameChanged(name) => {
//                 self.name = name;
//             }
//             Message::RoomIdChanged(room_id) => {
//                 self.room_id = room_id;
//             }
//             Message::SubmitCredsFrom => {
//             }
//         }
//     }

//     fn view(&self) -> Element<Message> {
//         let name_ip_field = TextInput::new("What is your name?", &self.name)
//             .on_input(Message::NameChanged);

//         let room_id_ip_field = TextInput::new("What is your room id?", &self.room_id)
//             .on_input(Message::RoomIdChanged);

//         let submit_btn = Button::new("Submit")
//             .on_press(Message::SubmitCredsFrom);

//         let layout = column![name_ip_field, room_id_ip_field, submit_btn];

//         layout.into()
//     }
// }


fn main() {

    let mut tcp_stream = TcpStream::connect("localhost:8000").expect("Failed to connect to server");
    let mut tcp_stream_write = tcp_stream.try_clone().expect("Failed to clone stream");

    let read_thread = thread::spawn(move || {

        loop {
            let mut buf = [0u8; 1024];
            tcp_stream.read(&mut buf).unwrap();
            if buf.is_empty() {
                break;
            }
            println!("{}", std::str::from_utf8(&buf).unwrap().trim());
        }
    });

    let write_thread = thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read input");

        let room_id_name = input.split(" ").collect::<Vec<&str>>();
        let _name = room_id_name[1].to_string().trim().to_string();
        let _room_id = room_id_name[0].to_string();

        tcp_stream_write.write_all(input.as_bytes()).expect("Failed to write to stream");

        loop {
            let mut input = String::new();
            stdin.read_line(&mut input).expect("Failed to read input");
            tcp_stream_write.write_all(input.as_bytes()).expect("Failed to write to stream");
        }
    });

    read_thread.join().unwrap();
    write_thread.join().unwrap();
}

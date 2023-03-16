use std::collections::HashMap;
use std::{
    net::TcpStream,
    sync::Mutex,
};
use lib::*;
use lib::models::NewUser;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;
use once_cell::sync::Lazy;
use tauri::{
    api::dialog::MessageDialogBuilder,
    State,
    Window,
    Manager
};
use serde_json::json;

// const SOCKET: &str = "als-kou.ddns.net:7878";
const SOCKET: &str = "127.0.0.1:7878";
static STREAM: Lazy<Mutex<TcpStream>> = Lazy::new(||{
    Mutex::new(TcpStream::connect(SOCKET).unwrap())
});

fn main() {
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("dfaddafaf"), 
            payload: String::new()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap());

    println!("{response:?}");
}

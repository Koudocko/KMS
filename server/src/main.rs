use std::{
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader},
    sync::{Mutex, Arc},
    thread, error::Error, fs::{OpenOptions, File},
};
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use lib::*;
use lib::models::{NewUser, User, NewKanji, NewVocab, NewGroup};
use chrono::Local;
use regex::Regex;

// const SOCKET: &str = "192.168.2.6:7878";
const SOCKET: &str = "127.0.0.1:7878";

fn log_activity(file: &Arc<Mutex<File>>, msg: String){
    let time = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    file.lock().unwrap().write_all(format!("{time} - {msg}\n\n").as_bytes()).unwrap();
    println!("{time} - {msg}\n");
}

fn handle_connection(stream: &mut (TcpStream, Option<User>), file: &Arc<Mutex<File>>)-> Eval<()>{
    stream.0.set_nonblocking(false)?;
    let request = read_stream(&mut stream.0)?;
    log_activity(file, format!("INCOMING REQUEST || From Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", stream.0.peer_addr()?.to_string(), stream.1.is_some(), request.header, request.payload));

    let mut header = String::from("GOOD");
    let payload = match request.header.as_str(){
        "GET_ACCOUNT_KEYS" =>{
            if let Some(keys) = get_account_keys(serde_json::from_str::<Value>(&request.payload)?)?{
                keys 
            }
            else{
                header = String::from("BAD");
                json!({ "error": "User does not exist! Please enter a valid username..." }).to_string()
            }
        }
        "VALIDATE_KEY" =>{
            if let Some(verify) = validate_key(serde_json::from_str::<Value>(&request.payload)?)?{
                if !verify.1{
                    header = String::from("BAD");
                    json!({ "error": "Password is invalid! Please re-enter your password..." }).to_string()
                }
                else{
                    stream.1 = Some(verify.0.clone());
                    String::new()
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Username does not exist! Please enter a valid username..." }).to_string()
            }
        }
        "CREATE_USER" =>{
            if create_user(serde_json::from_str::<NewUser>(&request.payload)?)?.is_none(){
               header = String::from("BAD");
               json!({ "error": "Username already exists! Please enter a different username..." }).to_string()
            }
            else{
                String::new()
            }
        }
        "CREATE_KANJI" =>{
            if let Some(user) = &stream.1{
                if create_kanji(&user, serde_json::from_str::<NewKanji>(&request.payload)?)?.is_none(){
                    header = String::from("BAD");
                    json!({ "error": "Kanji already exists in database!" }).to_string()
                }
                else{
                    String::new()
                }
            }
            else{
               return terminate::<()>();
            }
        }
        "CREATE_VOCAB" =>{
            if let Some(user) = &stream.1{
                if create_vocab(&user, serde_json::from_str::<NewVocab>(&request.payload)?)?.is_none(){
                    header = String::from("BAD");
                    json!({ "error": "Vocab already exists in database!" }).to_string()
                }
                else{
                    String::new()
                }
            }
            else{
               return terminate::<()>();
            }
        }
        "CREATE_GROUP" =>{
            if let Some(user) = &stream.1{
                let new_group = serde_json::from_str::<NewGroup>(&request.payload)?;

                if new_group.colour.is_none() || Regex::new(r"^#([0-9A-Fa-f]{6})$").unwrap().is_match(new_group.colour.as_ref().unwrap()){
                    if create_group(&user, new_group)?.is_none(){
                        header = String::from("BAD");
                        json!({ "error": "Group already exists in database!" }).to_string()
                    }
                    else{
                        String::new()
                    }
                }
                else{
                    header = String::from("BAD");
                    json!({ "error": "Provided colour is not a valid hexcode!}" }).to_string()
                }
            }
            else{
               return terminate::<()>();
            }
        }
        // "DELETE_USER" =>{
        //     if let Some(user) = &stream.1{
        //         if delete_user(&user, serde_json::from_str::<NewVocab>(&request.payload)?)?.is_none(){
        //             header = String::from("BAD");
        //             json!({ "error": "Vocab already exists in database!" }).to_string()
        //         }
        //         else{
        //             String::new()
        //         }
        //     }
        //     else{
        //        return terminate::<()>();
        //     }
        // }
        _ =>{
            header = String::from("BAD");
            json!({ "error": "Invalid request header!" }).to_string()
        }
    };

    let outgoing = Package{ header, payload };
    log_activity(&file, format!("OUTGOING REQUEST || To Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", stream.0.peer_addr()?.to_string(), stream.1.is_some(), outgoing.header, outgoing.payload));
    write_stream(&mut stream.0, outgoing)?;

    Ok(Some(()))
}

fn check_connections(streams: Arc<Mutex<Vec<(TcpStream, Option<User>)>>>, file: Arc<Mutex<File>>){
    loop{
        streams.lock().unwrap().retain_mut(|stream|{
            let mut buf = [0u8];
            stream.0.set_nonblocking(true).unwrap();
            if let Ok(peeked) = stream.0.peek(&mut buf){
                if peeked != 0{
                    if handle_connection(stream, &file).is_err(){
                        println!("CONNECTION TERMINATED || With Address: {}, Verified: {:?};", stream.0.peer_addr().unwrap().to_string(), stream.1.is_some());
                        stream.0.shutdown(std::net::Shutdown::Both).unwrap();
                        return false;
                    }
                }
            }

            true
        });
    }
}

fn main() {
    let file = Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/log/kms.log")
        .unwrap()));

    let listener = TcpListener::bind(SOCKET).unwrap();
    let streams = Arc::new(Mutex::new(Vec::new()));

    let stream_handle = Arc::clone(&streams);
    let file_handle = Arc::clone(&file);
    thread::spawn(||{
        check_connections(stream_handle, file_handle);
    });

    for stream in listener.incoming(){
        if let Ok(stream) = stream{
            log_activity(&file, format!("CONNECTION ESTABLISHED || With Address: {};", stream.peer_addr().unwrap().to_string()));
            streams.lock().unwrap().push((stream, None));
        }
        else{
            println!("FAILED TO ESTABLISH CONNECTION WITH CLIENT!");
        }
    }
}


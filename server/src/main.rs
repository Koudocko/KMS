use std::{
    io::{prelude::*, BufReader},
    sync::{Mutex, Arc, mpsc::channel},
    thread, error::Error, fs::{OpenOptions, File}, collections::HashMap, time::Duration, net::SocketAddr, ops::Index,
};
use serde_json::json;
use commands::*;
use lib::models::User;
use lib::Package;
use chrono::Local;
use threadpool::ThreadPool;
use tokio::{net::{TcpStream, TcpListener}, io::{AsyncReadExt, AsyncWriteExt}};

mod commands;

// const SOCKET: &str = "192.168.2.6:7878";
const SOCKET: &str = "127.0.0.1:7878";

fn log_activity(file: &Arc<Mutex<File>>, msg: String){
    let time = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    file.lock().unwrap().write_all(format!("{time} - {msg}\n\n").as_bytes()).unwrap();
    println!("{time} - {msg}\n");
}

fn handle_connection(user: &mut Option<User>, request: Package)-> Package{
    let mut header = String::from("GOOD");

    let payload = match request.header.as_str(){
        "GET_ACCOUNT_KEYS" =>{
            match get_account_keys(request.payload){
                Ok(keys) => keys,
                Err("INVALID_USER") =>{
                    header = String::from("BAD");
                    json!({ "error": "User does not exist! Please enter a valid username..." }).to_string()
                }
                Err("INVALID_FORMAT") =>{
                    header = String::from("BAD");
                    json!({ "error": "Request body format is ill-formed!" }).to_string()
                }
                _ => String::new(),
            }
        }
        "VALIDATE_KEY" =>{
            match validate_key(request.payload){
                Ok(verify) =>{
                    *user = Some(verify);
                    String::new()
                }
                Err("INVALID_USER") =>{
                    header = String::from("BAD");
                    json!({ "error": "Username does not exist! Please enter a valid username..." }).to_string()
                }
                Err("INVALID_FORMAT") =>{
                    header = String::from("BAD");
                    json!({ "error": "Request body format is ill-formed!" }).to_string()
                }
                Err("INVALID_PASSWORD") =>{
                    header = String::from("BAD");
                    json!({ "error": "Password is invalid! Please re-enter your password..." }).to_string()
                }
                _ => String::new(),
            }
        }
        "CREATE_USER" =>{
            if let Err("USER_EXISTS") = create_user(request.payload){
               header = String::from("BAD");
               json!({ "error": "Username already exists! Please enter a different username..." }).to_string()
            }
            else{
                String::new()
            }
        }
        "CREATE_KANJI" =>{
            if let Some(user) = user{
                match create_kanji(&user, request.payload){
                    Err("KANJI_EXISTS") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji already exists in database!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "CREATE_VOCAB" =>{
            if let Some(user) = user{
                match create_vocab(&user, request.payload){
                    Err("VOCAB_EXISTS") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab already exists in database!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "CREATE_GROUP" =>{
            if let Some(user) = user{
                match create_group(&user, request.payload){
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    Err("GROUP_EXISTS") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group already exists in database!" }).to_string()
                    }
                    Err("INVALID_HEXCODE") =>{
                        header = String::from("BAD");
                        json!({ "error": "Invalid format for hexcode! Provide a valid colour hexcode..." }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "CREATE_GROUP_KANJI" =>{
            if let Some(user) = user{
                match create_group_kanji(&user, request.payload){
                    Err("INVALID_KANJI") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji selected does not exist! Pick a valid Kanji..." }).to_string()
                    }
                    Err("INVALID_GROUP") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group selected does not exist! Pick a valid group..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    Err("ALREADY_ADDED") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji already added to group!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "CREATE_GROUP_VOCAB" =>{
            if let Some(user) = user{
                match create_group_vocab(&user, request.payload){
                    Err("INVALID_VOCAB") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab selected does not exist! Pick a valid vocab..." }).to_string()
                    }
                    Err("INVALID_GROUP") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group selected does not exist! Pick a valid group..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    Err("ALREADY_ADDED") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab already added to group!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_USER" =>{
            if let Some(user) = user{
                match delete_user(&user){
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_KANJI" =>{
            if let Some(user) = user{
                match delete_kanji(&user, request.payload){
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    Err("INVALID_KANJI") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji selected does not exist! Pick a valid kanji..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_VOCAB" =>{
            if let Some(user) = user{
                match delete_vocab(&user, request.payload){
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    Err("INVALID_VOCAB") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab selected does not exist! Pick a valid vocab..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_GROUP" =>{
            if let Some(user) = user{
                match delete_group(&user, request.payload){
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    Err("INVALID_GROUP") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group selected does not exist! Pick a valid gropu..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_GROUP_KANJI" =>{
            if let Some(user) = user{
                match delete_group_kanji(&user, request.payload){
                    Err("INVALID_KANJI") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji selected does not exist! Pick a valid Kanji..." }).to_string()
                    }
                    Err("INVALID_GROUP") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group selected does not exist! Pick a valid group..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    Err("ALREADY_REMOVED") =>{
                        header = String::from("BAD");
                        json!({ "error": "Kanji already removed from group!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        "DELETE_GROUP_VOCAB" =>{
            if let Some(user) = user{
                match delete_group_vocab(&user, request.payload){
                    Err("INVALID_VOCAB") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab selected does not exist! Pick a valid vocab..." }).to_string()
                    }
                    Err("INVALID_GROUP") =>{
                        header = String::from("BAD");
                        json!({ "error": "Group selected does not exist! Pick a valid group..." }).to_string()
                    }
                    Err("INVALID_FORMAT") =>{
                        header = String::from("BAD");
                        json!({ "error": "Request body format is ill-formed!" }).to_string()
                    }
                    Err("ALREADY_REMOVED") =>{
                        header = String::from("BAD");
                        json!({ "error": "Vocab already removed from group!" }).to_string()
                    }
                    Err("INVALID_USER") =>{
                        header = String::from("BAD");
                        json!({ "error": "User has been invalidated!" }).to_string()
                    }
                    _ => String::new(),
                }
            }
            else{
                header = String::from("BAD");
                json!({ "error": "Unverified request! Login to a valid account to make this request..." }).to_string()
            }
        }
        _ =>{
            header = String::from("BAD");
            json!({ "error": "Invalid request header!" }).to_string()
        }
    };

   Package{ id: request.id, header, payload }
}

async fn check_connection(mut stream: TcpStream, addr: SocketAddr, file_handle: Arc<Mutex<File>>){
    let mut buf = [0_u8; 4096];
    let mut user = None::<User>;

    loop{
        match stream.read(&mut buf).await{
            Ok(0) =>{
                log_activity(&file_handle, format!("CONNECTION TERMINATED NORMALLY || With Address: {}, User: {:?};", 
                    addr.to_string(),
                    user));
                return;
            }
            Ok(_) =>{
                let mut response = Package{
                    id: -1,
                    header: String::from("BAD"),
                    payload: json!({ "error": "Request body format is ill-formed!" }).to_string(),
                };

                if let Some(package_end) = buf.iter().position(|x| *x == b'\n'){
                    if let Ok(request) = serde_json::from_slice::<Package>(&buf[..package_end]){
                        log_activity(&file_handle, format!("INCOMING REQUEST || From Address: {}, User: {:?}, Header: {}, Payload: {:?};", 
                            addr.to_string(),
                            user, 
                            request.header, 
                            request.payload));
                        response = handle_connection(&mut user, request);
                    }
                }

                let mut response_bytes = serde_json::to_vec(&response).unwrap();
                response_bytes.push(b'\n');

                if stream.write_all(&response_bytes).await.is_ok(){
                    log_activity(&file_handle, format!("OUTGOING RESPONSE SENT || To Address: {}, User: {:?}, Header: {}, Payload: {:?};", 
                        addr.to_string(),
                        user,
                        response.header, 
                        response.payload));
                }
                else{
                    log_activity(&file_handle, format!("OUTGOING RESPONSE FAILED || To Address: {}, User: {:?}, Header: {}, Payload: {:?};", 
                        addr.to_string(),
                        user,
                        response.header, 
                        response.payload));
                }
            }
            Err(_) =>{
                log_activity(&file_handle, format!("CONNECTION TERMINATED ABNORMALLY || With Address: {}, User: {:?};", 
                    addr.to_string(),
                    user));
                return;
            }
        }
    }
}

#[tokio::main]
async fn main(){
    let file = Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/log/kms.log")
        .unwrap()));

    let listener = TcpListener::bind(SOCKET).await.unwrap();


    loop{
        let file_handle = Arc::clone(&file);

        if let Ok((stream, addr)) = listener.accept().await{
            log_activity(&file, format!("CONNECTION ESTABLISHED || With Address: {};", 
                stream.peer_addr().unwrap().to_string()));
            tokio::spawn(check_connection(stream, addr, file_handle));
        }
        else{
            println!("FAILED TO ESTABLISH CONNECTION WITH CLIENT!");
        }
    }
}

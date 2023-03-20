use std::{
    net::{TcpListener, TcpStream},
    io::prelude::*,
    sync::{Mutex, Arc},
    thread, error::Error, fs::{OpenOptions, File},
};
use serde_json::json;
use lib::*;
use lib::models::User;
use chrono::Local;

// const SOCKET: &str = "192.168.2.6:7878";
const SOCKET: &str = "127.0.0.1:7878";

fn log_activity(file: &Arc<Mutex<File>>, msg: String){
    let time = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    file.lock().unwrap().write_all(format!("{time} - {msg}\n\n").as_bytes()).unwrap();
    println!("{time} - {msg}\n");
}

fn handle_connection(stream: &mut (TcpStream, Option<User>), file: &Arc<Mutex<File>>)-> Result<(), Box<dyn Error>>{
    stream.0.set_nonblocking(false)?;
    let request = read_stream(&mut stream.0)?;
    log_activity(file, format!("INCOMING REQUEST || From Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", 
            stream.0.peer_addr()?.to_string(), 
            stream.1.is_some(), 
            request.header, 
            request.payload));

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
                    if !verify.1{
                        header = String::from("BAD");
                        json!({ "error": "Password is invalid! Please re-enter your password..." }).to_string()
                    }
                    else{
                        stream.1 = Some(verify.0.clone());
                        String::new()
                    }
                }
                Err("INVALID_USER") =>{
                    header = String::from("BAD");
                    json!({ "error": "Username does not exist! Please enter a valid username..." }).to_string()
                }
                Err("INVALID_FORMAT") =>{
                    header = String::from("BAD");
                    json!({ "error": "Request body format is ill-formed!" }).to_string()
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
            if let Some(user) = &stream.1{
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
        _ =>{
            header = String::from("BAD");
            json!({ "error": "Invalid request header!" }).to_string()
        }
    };

    let outgoing = Package{ header, payload };
    log_activity(&file, format!("OUTGOING REQUEST || To Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", 
            stream.0.peer_addr()?.to_string(), 
            stream.1.is_some(), 
            outgoing.header, 
            outgoing.payload));
    write_stream(&mut stream.0, outgoing)?;

    Ok(())
}

fn check_connections(streams: Arc<Mutex<Vec<(TcpStream, Option<User>)>>>, file: Arc<Mutex<File>>){
    loop{
        streams.lock().unwrap().retain_mut(|stream|{
            let mut buf = [0u8];
            stream.0.set_nonblocking(true).unwrap();
            if let Ok(peeked) = stream.0.peek(&mut buf){
                if peeked != 0{
                    if handle_connection(stream, &file).is_err(){
                        println!("CONNECTION TERMINATED || With Address: {}, Verified: {:?};", 
                            stream.0.peer_addr().unwrap().to_string(), 
                            stream.1.is_some());
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
            log_activity(&file, format!("CONNECTION ESTABLISHED || With Address: {};", 
                stream.peer_addr().unwrap().to_string()));
            streams.lock().unwrap().push((stream, None));
        }
        else{
            println!("FAILED TO ESTABLISH CONNECTION WITH CLIENT!");
        }
    }
}


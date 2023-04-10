use std::{
    net::{TcpListener, TcpStream},
    io::prelude::*,
    sync::{Mutex, Arc, mpsc::channel},
    thread, error::Error, fs::{OpenOptions, File}, collections::HashMap, time::Duration,
};
use serde_json::json;
use lib::*;
use lib::models::User;
use chrono::Local;
use threadpool::ThreadPool;

// const SOCKET: &str = "192.168.2.6:7878";
const SOCKET: &str = "127.0.0.1:7878";

fn log_activity(file: &Arc<Mutex<File>>, msg: String){
    let time = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    file.lock().unwrap().write_all(format!("{time} - {msg}\n\n").as_bytes()).unwrap();
    println!("{time} - {msg}\n");
}

fn handle_connection(stream: &mut TcpStream, user: &Option<User>, file: &Arc<Mutex<File>>)-> Result<Option<User>, Box<dyn Error>>{
    stream.set_nonblocking(false)?;
    let request = read_stream(stream)?;
    log_activity(file, format!("INCOMING REQUEST || From Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", 
            stream.peer_addr()?.to_string(), 
            user.is_some(), 
            request.header, 
            request.payload));

    let mut new_user = user.clone();
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
                    new_user = Some(verify);
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

    let outgoing = Package{ header, payload };
    log_activity(&file, format!("OUTGOING REQUEST || To Address: {}, Verified: {:?}, Header: {}, Payload: {:?};", 
            stream.peer_addr()?.to_string(), 
            new_user.is_some(), 
            outgoing.header, 
            outgoing.payload));
    write_stream(stream, outgoing)?;

    Ok(new_user)
}

fn check_connections(streams: Arc<Mutex<HashMap<Option<User>, Vec<Arc<Mutex<TcpStream>>>>>>, file: Arc<Mutex<File>>){
    let pool = Arc::new(ThreadPool::new(7));
    let (tx_connection, rx_connection) = channel();

    loop{
        let mut mutex_handle = streams.lock().unwrap();
        let streams_len = mutex_handle.len();

        for (user, streams) in mutex_handle.iter_mut(){
            let user = Arc::new(user.clone());

            for (idx, stream) in streams.iter_mut().enumerate(){
                let tx_connection = tx_connection.clone();
                let file = Arc::clone(&file);
                let stream = Arc::clone(&stream);
                let user = Arc::clone(&user);
                
                pool.execute(move ||{
                    let mut stream_guard = stream.lock().unwrap();
                    let mut buf = [0u8];

                    if let Ok(peeked) = stream_guard.peek(&mut buf){
                        if peeked != 0{
                            if let Ok(new_user) = handle_connection(&mut (*stream_guard), &user, &file){
                                if new_user != *user{
                                    tx_connection.send(Some((((*user).clone(), idx), Some((new_user, Arc::clone(&stream)))))).unwrap();
                                }
                                else{
                                    tx_connection.send(None).unwrap();
                                }

                                return;
                            }
                        }
                    }
                    else{
                        tx_connection.send(None).unwrap();
                        return;
                    }

                    println!("CONNECTION TERMINATED || With Address: {}, Verified: {:?};", 
                        stream_guard.peer_addr().unwrap().to_string(), 
                        user.is_some());
                    stream_guard.shutdown(std::net::Shutdown::Both).unwrap();

                    tx_connection.send(Some((((*user).clone(), idx), None))).unwrap();
                });
            }
        }

        let mut broken_connections = Vec::new();
        for _ in 0..streams_len{
            broken_connections.push(rx_connection.recv().unwrap());
        }

        let mut broken_connections = broken_connections.into_iter().filter_map(|ele|{
            if let Some(data) = ele{
                return Some(data);
            }
            None
        }).collect::<Vec<((Option<User>, usize), Option<(Option<User>, Arc<Mutex<TcpStream>>)>)>>();
        broken_connections.sort_by_key(|key| key.0.1);

        for broken_connection in broken_connections.into_iter().rev(){
            if let Some(new_user) = broken_connection.1{
                if let Some(new_user_connections) = mutex_handle.get_mut(&new_user.0){
                    new_user_connections.push(new_user.1);
                }
                else{
                    mutex_handle.insert(new_user.0, vec![new_user.1]);
                }
            }

            let user_connections = mutex_handle.get_mut(&broken_connection.0.0).unwrap();
            user_connections.remove(broken_connection.0.1);

            if user_connections.is_empty(){
                mutex_handle.remove(&broken_connection.0.0);
            }
        }
    }
}

fn main() {
    let file = Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/log/kms.log")
        .unwrap()));

    let listener = TcpListener::bind(SOCKET).unwrap();
    let streams = Arc::new(Mutex::new(HashMap::new()));

    let stream_handle = Arc::clone(&streams);
    let file_handle = Arc::clone(&file);
    thread::spawn(||{
        check_connections(stream_handle, file_handle);
    });

    for stream in listener.incoming(){
        if let Ok(stream) = stream{
            log_activity(&file, format!("CONNECTION ESTABLISHED || With Address: {};", 
                stream.peer_addr().unwrap().to_string()));

            stream.set_read_timeout(Some(Duration::from_nanos(1))).unwrap();
            streams.lock().unwrap().insert(None, vec![Arc::new(Mutex::new(stream))]);
            println!("PUSHED");
        }
        else{
            println!("FAILED TO ESTABLISH CONNECTION WITH CLIENT!");
        }
    }
}

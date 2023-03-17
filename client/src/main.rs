use std::collections::HashMap;
use std::{
    net::TcpStream,
    sync::Mutex,
};
use lib::*;
use lib::models::{NewUser, NewKanji, NewVocab};
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

fn add_vocab(phrase: String, meaning: String, reading: Vec<Option<String>>, description: Option<String>){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_VOCAB"), 
            payload: serde_json::to_string(&NewVocab{
                phrase,
                meaning,
                reading,
                description,
                kanji_refs: Vec::new(),
                user_id: 0,
            }).unwrap()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("ADDED VOCAB");
    }
    else{
        println!("FAILLED ADD");
    }
}

fn add_kanji(symbol: String, meaning: String, onyomi: Vec<Option<String>>, kunyomi: Vec<Option<String>>, description: Option<String>){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_KANJI"), 
            payload: serde_json::to_string(&NewKanji{
                symbol,
                meaning,
                onyomi,
                kunyomi,
                description,
                vocab_refs: Vec::new(),
                user_id: 0,
            }).unwrap()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("ADDED KANJI");
    }
    else{
        println!("FAILLED ADD");
    }
}

fn login_user(username: String, password: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("GET_ACCOUNT_KEYS"), 
            payload: json!({ "username": username }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    if response.header == "GOOD"{
        const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
        let n_iter = NonZeroU32::new(100_000).unwrap();
        
        let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
        let salt_key = unpack(&response.payload, "salt")
            .as_array()
            .unwrap()
            .into_iter()
            .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
            .collect::<Vec<u8>>();

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            n_iter,
            &salt_key,
            password.as_bytes(),
            &mut pbkdf2_hash,
        );

        write_stream(&mut *STREAM.lock().unwrap(), 
            Package { 
                header: String::from("VALIDATE_KEY"), 
                payload: json!({ "username": username, "hash": pbkdf2_hash.to_vec() }).to_string()
            }
        ).unwrap();

        let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
        if response.header == "GOOD"{
            println!("SIGNED IN");
        }
        else{
            println!("ERROR");
        }
    }
    else{
        println!("ERROR");
    }
}

fn add_user(username: String, password: (String, String)){
    if password.0 == password.1{
        const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
        let n_iter = NonZeroU32::new(100_000).unwrap();
        let rng = rand::SystemRandom::new();

        let mut salt_key = [0u8; CREDENTIAL_LEN];
        rng.fill(&mut salt_key).unwrap();

        let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            n_iter,
            &salt_key,
            password.0.as_bytes(),
            &mut pbkdf2_hash,
        );
        
        let account = NewUser{ 
            username: username.to_owned(), 
            hash: pbkdf2_hash.to_vec(), 
            salt: salt_key.to_vec(),
        };

        write_stream(&mut *STREAM.lock().unwrap(), 
            Package { 
                header: String::from("CREATE_USER"), 
                payload: serde_json::to_string(&account).unwrap()
            }
        ).unwrap();

        let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
        if response.header == "GOOD"{
            println!("ACCOUNT CREATED");
        }
        else{
            println!("ERROR");
        }
    }
    else{
        println!("ERROR");
    }
}

fn main(){
    // login_user("Joe biden".to_owned(), "__joebidengaming64___".to_owned());
    // add_user("Joe biden".to_owned(), ("__joebidengaming64___".to_owned(), "__joebidengaming64___".to_owned()));
    // add_kanji(String::from("女"), String::from("Woman"), vec![Some(String::from("じょ"))], vec![Some(String::from("おんな"))], Some(String::from("Jolyne the woman.")));
}

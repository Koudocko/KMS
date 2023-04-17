use std::collections::HashMap;
use std::{
    net::TcpStream,
    sync::Mutex,
};
use lib::*;
use lib::models::{NewUser, NewKanji, NewVocab, NewGroup, Group, Kanji};
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

// fn get_kanji()-> (Option<Group>, Vec<Kanji>){
    
// }

fn change_group(group_title: String, group_colour: String, members_removed: Vec<String>){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("EDIT_GROUP"), 
            payload: json!({ "group_title": group_title, "group_colour": group_colour, "members_removed": members_removed }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("CHANGED GROUP");
    }
    else{
        println!("FAILLED CHANGE");
    }
}

fn remove_group_vocab(vocab_phrase: String, group_title: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_GROUP_VOCAB"), 
            payload: json!({ "vocab_phrase": vocab_phrase, "group_title": group_title }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED vocab from GROUP");
    }
    else{
        println!("FAILLED remove");
    }
}

fn remove_group_kanji(kanji_symbol: String, group_title: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_GROUP_KANJI"), 
            payload: json!({ "kanji_symbol": kanji_symbol, "group_title": group_title }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED Kanji from GROUP");
    }
    else{
        println!("FAILLED remove");
    }
}

fn remove_group(group_title: String, group_vocab: bool){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_GROUP"), 
            payload: json!({ "group_title": group_title, "group_vocab": group_vocab }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED GROUP");
    }
    else{
        println!("FAILLED REMOVE");
    }
}

fn remove_vocab(vocab_phrase: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_VOCAB"), 
            payload: json!({ "vocab_phrase": vocab_phrase }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED VOCAB");
    }
    else{
        println!("FAILLED REMOVE");
    }
}

fn remove_kanji(kanji_symbol: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_KANJI"), 
            payload: json!({ "kanji_symbol": kanji_symbol }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED KANJI");
    }
    else{
        println!("FAILLED REMOVE");
    }
}

fn remove_user(){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("DELETE_USER"), 
            payload: String::new()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("REMOVED USER");
    }
    else{
        println!("FAILLED REMOVE");
    }
}

fn add_group_vocab(vocab_phrase: String, group_title: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_GROUP_VOCAB"), 
            payload: json!({ "vocab_phrase": vocab_phrase, "group_title": group_title }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("ADDED vocab to GROUP");
    }
    else{
        println!("FAILLED ADD");
    }
}

fn add_group_kanji(kanji_symbol: String, group_title: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_GROUP_KANJI"), 
            payload: json!({ "kanji_symbol": kanji_symbol, "group_title": group_title }).to_string()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("ADDED Kanji to GROUP");
    }
    else{
        println!("FAILLED ADD");
    }
}

fn add_group(group_title: String, group_colour: Option<String>, group_vocab: bool){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_GROUP"), 
            payload: serde_json::to_string(&NewGroup{
                title: group_title,
                colour: group_colour,
                vocab: group_vocab,
                user_id: 0,
            }).unwrap()
        }
    ).unwrap();

    let response = read_stream(&mut *STREAM.lock().unwrap()).unwrap();
    
    if response.header == "GOOD"{
        println!("ADDED GROUP");
    }
    else{
        println!("FAILLED ADD");
    }
}

fn add_vocab(vocab_phrase: String, vocab_meaning: String, vocab_reading: Vec<Option<String>>, vocab_description: Option<String>){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_VOCAB"), 
            payload: serde_json::to_string(&NewVocab{
                phrase: vocab_phrase,
                meaning: vocab_meaning,
                reading: vocab_reading,
                description: vocab_description,
                kanji_refs: Vec::new(),
                user_id: 0,
                group_id: None,
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

fn add_kanji(kanji_symbol: String, kanji_meaning: String, kanji_onyomi: Vec<Option<String>>, kanji_kunyomi: Vec<Option<String>>, kanji_description: Option<String>){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("CREATE_KANJI"), 
            payload: serde_json::to_string(&NewKanji{
                symbol: kanji_symbol,
                meaning: kanji_meaning,
                onyomi: kanji_onyomi,
                kunyomi: kanji_kunyomi,
                description: kanji_description,
                vocab_refs: Vec::new(),
                user_id: 0,
                group_id: None,
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

fn login_user(user_username: String, user_password: String){
    write_stream(&mut *STREAM.lock().unwrap(), 
        Package { 
            header: String::from("GET_ACCOUNT_KEYS"), 
            payload: json!({ "user_username": user_username }).to_string()
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
            user_password.as_bytes(),
            &mut pbkdf2_hash,
        );

        write_stream(&mut *STREAM.lock().unwrap(), 
            Package { 
                header: String::from("VALIDATE_KEY"), 
                payload: json!({ "user_username": user_username, "user_hash": pbkdf2_hash.to_vec() }).to_string()
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

fn add_user(user_username: String, user_password: (String, String)){
    if user_password.0 == user_password.1{
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
            user_password.0.as_bytes(),
            &mut pbkdf2_hash,
        );
        
        let account = NewUser{ 
            username: user_username.to_owned(), 
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
    // add_user("Joe biden".to_owned(), ("__joebidengaming64___".to_owned(), "__joebidengaming64___".to_owned()));
    // login_user("Joe biden".to_owned(), "__joebidengaming64___".to_owned());
    loop{
        login_user("Joe biden".to_owned(), "__joebidengaming64___".to_owned());
    }
    // add_group("Nouns".to_owned(), Some("#FFFFFF".to_owned()), false);
    // add_kanji(String::from("女"), String::from("Woman"), vec![Some(String::from("じょ"))], vec![Some(String::from("おんな"))], Some(String::from("Jolyne the woman.")));
    // add_kanji(String::from("下"), String::from("Down"), vec![Some(String::from("か")), Some(String::from("げ"))], vec![Some(String::from("した")), Some(String::from("くだ")), Some(String::from("さ")), Some(String::from("お"))], Some(String::from("Below the sh*t under my toe, I look down and see a car and its keys.")));
    // add_vocab(String::from("下さい"), String::from("Please"), vec![Some(String::from("ください"))], Some(String::from("Kudos, you got it correct now please leave.")));
    // add_group_kanji(String::from("女"), String::from("Nouns"));
    // add_group_vocab(String::from("下さい"), String::from("Nouns"));
    // remove_kanji(String::from("女"));
    // remove_vocab(String::from("下さい"));
    // remove_group(String::from("Nouns"), false);
    // remove_user();
    // remove_group_kanji(String::from("女"), String::from("Nouns"));
    // remove_group_vocab(String::from("下さい"), String::from("Nouns"));
}

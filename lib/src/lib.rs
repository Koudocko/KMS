use serde::{Serialize, Deserialize};
use schema::{*, groups::user_id};
use std::{
    io::{prelude::*, BufReader, self},
    net::TcpStream,
};
use diesel::{
    pg::PgConnection,
    prelude::*,
};
use models::*;
use serde_json::{json, Value};
use regex::Regex;

pub mod schema;
pub mod models;

pub type Eval<T> = Result<T, &'static str>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Package{
    pub header: String,
    pub payload: String
}

pub fn unpack(payload: &str, field: &str)-> Value{
    serde_json::from_str::<Value>(payload).unwrap()[field].clone()
}

pub fn write_stream(stream: &mut TcpStream, package: Package)-> Result<(), io::Error>{
    let mut buf: Vec<u8> = serde_json::to_vec(&package)?;
    buf.push(b'\n');
    stream.write_all(&mut buf)?;

    Ok(())
}

pub fn read_stream(stream: &mut TcpStream)-> Result<Package, io::Error>{
    let mut buf = String::new();

    BufReader::new(stream)
        .read_line(&mut buf)?;

    Ok(serde_json::from_str(&buf)?)
}

pub fn establish_connection() -> PgConnection{
    let database_url = "postgres://postgres@localhost/kms";

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_user(payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if let Ok(payload) = serde_json::from_str::<NewUser>(&payload){
        if users::table.filter(users::username.eq(payload.username.to_owned()))
            .first::<User>(connection).is_err(){
            diesel::insert_into(users::table)
                .values(&payload)
                .execute(connection)
                .is_ok();

            return Ok(());
        }
        
        return Err("USER_EXISTS");
    }

    Err("INVALID_FORMAT")
}

pub fn get_account_keys(payload: String)-> Eval<String>{
    let connection = &mut establish_connection();

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(payload) = payload["username"].as_str(){
            if let Ok(user) = users::table.filter(users::username.eq(payload))
                .first::<User>(connection){
                return Ok(json!({ "salt": user.salt }).to_string());
            }

            return Err("INVALID_USER");
        }
    }

    Err("INVALID_FORMAT")
}

pub fn validate_key(payload: String)-> Eval<(User, bool)>{
    let connection = &mut establish_connection();

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(user_hash) = payload["hash"].as_array(){
            let user_hash = user_hash.into_iter().map(|byte|{
                if let Some(byte) = byte.as_u64(){
                    if let Ok(byte) = u8::try_from(byte){
                        return byte
                    }
                }

                0
            }).collect::<Vec<u8>>();

            if let Some(user_username) = payload["username"].as_str(){
                if let Ok(user) = users::table.filter(users::username.eq(user_username))
                    .first::<User>(connection){
                    let mut idx = 0;
                    let verified = !user_hash.iter().any(|byte|{
                        let check = *byte != user.hash[idx];
                        idx += 1;
                        check
                    });

                    return Ok((user, verified));
                }

                return Err("INVALID_USER");
            }
        }
    }

    Err("INVALID_FORMAT")
}

pub fn create_kanji(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(mut payload) = serde_json::from_str::<NewKanji>(&payload){
        if kanji::table.filter(kanji::symbol.eq(&payload.symbol))
            .filter(kanji::user_id.eq(user.id))
            .first::<Kanji>(connection).is_err(){
            payload.user_id = user.id;

            for mut vocab in Vocab::belonging_to(&user)
                .load::<Vocab>(connection)
                .unwrap(){
                if vocab.phrase.contains(&payload.symbol){
                    vocab.kanji_refs.push(Some(payload.symbol.to_owned()));

                    diesel::update(&vocab)
                        .set(vocab::kanji_refs.eq(&vocab.kanji_refs))
                        .execute(connection)
                        .is_ok();

                    payload.vocab_refs.push(Some(vocab.phrase));
                }
            }

            diesel::insert_into(kanji::table)
                .values(&payload)
                .execute(connection)
                .is_ok();

            return Ok(());
        }
        
        return Err("KANJI_EXISTS");
    }

    Err("INVALID_FORMAT")
}

pub fn create_vocab(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(mut payload) = serde_json::from_str::<NewVocab>(&payload){
        if vocab::table.filter(vocab::phrase.eq(&payload.phrase))
            .filter(vocab::user_id.eq(user.id))
            .first::<Vocab>(connection).is_err(){
            payload.user_id = user.id;

            for kanji in payload.phrase.chars(){
               if let Ok(mut kanji) = kanji::table.filter(kanji::symbol.eq(kanji.to_string())) 
                   .filter(kanji::user_id.eq(user.id))
                   .first::<Kanji>(connection){
                    kanji.vocab_refs.push(Some(payload.phrase.to_owned()));

                    diesel::update(&kanji)
                        .set(kanji::vocab_refs.eq(&kanji.vocab_refs))
                        .execute(connection)
                        .is_ok();

                    payload.kanji_refs.push(Some(kanji.symbol));
               }
            }

            diesel::insert_into(vocab::table)
                .values(&payload)
                .execute(connection)
                .is_ok();

            return Ok(());
        }
        
       return  Err("VOCAB_EXISTS");
    }

    Err("INVALID_FORMAT")
}

pub fn create_group(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(mut payload) = serde_json::from_str::<NewGroup>(&payload){
        if payload.colour.is_none() || Regex::new(r"^#([0-9A-Fa-f]{6})$")
            .unwrap()
            .is_match(payload.colour.as_ref()
                .unwrap()){
            if groups::table.filter(groups::title.eq(&payload.title))
                .filter(groups::user_id.eq(user.id))
                .filter(groups::vocab.eq(payload.vocab))
                .first::<Group>(connection).is_err(){
                payload.user_id = user.id;

                diesel::insert_into(groups::table)
                    .values(&payload)
                    .execute(connection)
                    .is_ok();

                return Ok(());
            }
            
            return Err("GROUP_EXISTS");
        }

        return Err("INVALID_HEXCODE");
    }

    Err("INVALID_FORMAT")
}

pub fn create_group_kanji(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(group_title) = payload["group"].as_str(){
            if let Some(kanji_symbol) = payload["kanji"].as_str(){
                if let Ok(user_group) = groups::table.filter(groups::title.eq(group_title))
                    .filter(groups::user_id.eq(user.id))
                    .filter(groups::vocab.eq(false))
                    .first::<Group>(connection){
                    if let Ok(user_kanji) = kanji::table.filter(kanji::symbol.eq(kanji_symbol))
                        .filter(kanji::user_id.eq(user.id))
                        .first::<Kanji>(connection){

                        if user_kanji.group_id.is_none(){
                            diesel::update(&user_kanji)
                                .set(kanji::group_id.eq(user_group.id))
                                .execute(connection)
                                .is_ok();

                            return Ok(());
                        }

                        return Err("ALREADY_ADDED");
                    }

                    return Err("INVALID_KANJI")
                }

                return Err("INVALID_GROUP")
            }
        }
    }

    Err("INVALID_FORMAT")
}

pub fn delete_user(user: &User)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    for kanji in Kanji::belonging_to(user)
        .load::<Kanji>(connection)
        .unwrap(){
        diesel::delete(&kanji)
            .execute(connection)
            .is_ok();
    }

    for vocab in Vocab::belonging_to(user)
        .load::<Vocab>(connection)
        .unwrap(){
        diesel::delete(&vocab)
            .execute(connection)
            .is_ok();
    }

    for group in Group::belonging_to(user)
        .load::<Group>(connection)
        .unwrap(){
        diesel::delete(&group)
            .execute(connection)
            .is_ok();
    }

    diesel::delete(user)
        .execute(connection)
        .is_ok();

    Ok(())
}

pub fn delete_kanji(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(kanji_symbol) = payload["kanji"].as_str(){
            if let Ok(user_kanji) = kanji::table.filter(kanji::symbol.eq(kanji_symbol))
                .filter(kanji::user_id.eq(user.id))
                .first::<Kanji>(connection){
                diesel::delete(&user_kanji)
                    .execute(connection)
                    .is_ok();

                return Ok(());
            }

            return Err("INVALID_KANJI");
        }
    }

    Err("INVALID_FORMAT")
}

pub fn delete_vocab(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(vocab_phrase) = payload["vocab"].as_str(){
            if let Ok(user_vocab) = vocab::table.filter(vocab::phrase.eq(vocab_phrase))
                .filter(vocab::user_id.eq(user.id))
                .first::<Vocab>(connection){
                diesel::delete(&user_vocab)
                    .execute(connection)
                    .is_ok();

                return Ok(());
            }

            return Err("INVALID_VOCAB");
        }
    }

    Err("INVALID_FORMAT")
}

pub fn delete_group(user: &User, payload: String)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.find(user.id)
        .first::<User>(connection).is_err(){
        return Err("INVALID_USER")
    }

    if let Ok(payload) = serde_json::from_str::<Value>(&payload){
        if let Some(group_title) = payload["group"].as_str(){
            if let Ok(user_group) = groups::table.filter(groups::title.eq(group_title))
                .filter(groups::user_id.eq(user.id))
                .first::<Group>(connection){
                if user_group.vocab{
                    for vocab in Vocab::belonging_to(&user_group)
                        .load::<Vocab>(connection)
                        .unwrap(){
                        diesel::update(&vocab)
                            .set(vocab::group_id.eq(None::<i32>))
                            .execute(connection)
                            .is_ok();
                    }
                }
                else{
                    for kanji in Kanji::belonging_to(&user_group)
                        .load::<Kanji>(connection)
                        .unwrap(){
                        diesel::update(&kanji)
                            .set(kanji::group_id.eq(None::<i32>))
                            .execute(connection)
                            .is_ok();
                    }
                }

                diesel::delete(&user_group)
                    .execute(connection)
                    .is_ok();

                return Ok(());
            }

            return Err("INVALID_GROUP");
        }
    }

    Err("INVALID_FORMAT")
}

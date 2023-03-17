use serde::{Serialize, Deserialize};
use schema::*;
use std::{
    io::{prelude::*, BufReader},
    net::TcpStream, error::Error
};
use diesel::{
    pg::PgConnection,
    prelude::*,
};
use models::*;
use serde_json::{json, Value};
use std::fmt;

pub mod schema;
pub mod models;

#[derive(Serialize, Deserialize, Debug)]
pub struct Package{
    pub header: String,
    pub payload: String
}

#[derive(Debug, Clone)]
pub struct PlainError;
impl PlainError{
    pub fn new()-> PlainError{ PlainError }
}
impl fmt::Display for PlainError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
impl Error for PlainError{}

pub fn unpack(payload: &str, field: &str)-> Value{
    serde_json::from_str::<Value>(payload).unwrap()[field].clone()
}

pub fn write_stream(stream: &mut TcpStream, package: Package)-> Result<(), std::io::Error>{
    let mut buf: Vec<u8> = serde_json::to_vec(&package)?;
    buf.push(b'\n');
    stream.write_all(&mut buf)?;

    Ok(())
}

pub fn read_stream(stream: &mut TcpStream)-> Result<Package, std::io::Error>{
    let mut buf = String::new();

    BufReader::new(stream)
        .read_line(&mut buf)?;

    Ok(serde_json::from_str(&buf)?)
}

pub fn establish_connection() -> PgConnection {
    let database_url = "postgres://postgres@localhost/kms";

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_user(payload: NewUser)-> bool{
    let connection = &mut establish_connection();

    if users::table.filter(users::username.eq(payload.username.to_owned()))
        .first::<User>(connection).is_err(){
        diesel::insert_into(users::table)
            .values(&payload)
            .execute(connection)
            .unwrap();

        return true;
    }
    
    false
}

pub fn get_account_keys(payload: Value)-> Result<Option<String>, Box<dyn Error>>{
    let connection = &mut establish_connection();

    if let Some(payload) = payload["username"].as_str(){
        if let Ok(user) = users::table.filter(users::username.eq(payload))
            .first::<User>(connection){
            Ok(Some(json!({ "salt": user.salt }).to_string()))
        }
        else{
            Ok(None)
        }
    }
    else{
       Err(Box::new(PlainError::new()))
    }
}

pub fn validate_key(payload: Value)-> Result<Option<(User, bool)>, Box<dyn Error>>{
    let connection = &mut establish_connection();

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
            if let Ok(user) = users::table.filter(users::username.eq(user_username)).first::<User>(connection){
                let mut idx = 0;
                let verified = !user_hash.iter().any(|byte|{
                    let check = *byte != user.hash[idx];
                    idx += 1;
                    check
                });

                return Ok(Some((user, verified)));
            }
            else{
                return Ok(None);
            }
        }
    }

   Err(Box::new(PlainError::new()))
}

pub fn create_kanji(user: &User, mut payload: NewKanji)-> bool{
    let connection = &mut establish_connection();

    if kanji::table.filter(kanji::symbol.eq(&payload.symbol))
        .filter(kanji::user_id.eq(user.id))
        .first::<Kanji>(connection).is_err(){
        payload.user_id = user.id;

        Vocab::belonging_to(&user)
            .load::<Vocab>(connection)
            .unwrap()
            .into_iter()
            .for_each(|mut vocab|{
                if vocab.phrase.contains(&payload.symbol){
                    vocab.kanji_refs.push(Some(payload.symbol.to_owned()));

                    diesel::update(vocab::table.find(vocab.id))
                        .set(vocab::kanji_refs.eq(vocab.kanji_refs))
                        .execute(connection)
                        .unwrap();

                    payload.vocab_refs.push(Some(vocab.phrase));
                }
            });


        diesel::insert_into(kanji::table)
            .values(&payload)
            .execute(connection)
            .unwrap();

        return true;
    }
    
    false
}

pub fn create_vocab(user: &User, mut payload: NewVocab)-> bool{
    let connection = &mut establish_connection();

    if vocab::table.filter(vocab::phrase.eq(&payload.phrase))
        .filter(vocab::user_id.eq(user.id))
        .first::<Vocab>(connection).is_err(){
        payload.user_id = user.id;

        for kanji in payload.phrase.chars(){
           if let Ok(mut kanji) = kanji::table.filter(kanji::symbol.eq(kanji.to_string())) 
               .filter(kanji::user_id.eq(user.id))
               .first::<Kanji>(connection){
                kanji.vocab_refs.push(Some(payload.phrase.to_owned()));

                diesel::update(kanji::table.find(kanji.id))
                    .set(kanji::vocab_refs.eq(kanji.vocab_refs))
                    .execute(connection)
                    .unwrap();

                payload.kanji_refs.push(Some(kanji.symbol));
           }
        }

        diesel::insert_into(vocab::table)
            .values(&payload)
            .execute(connection)
            .unwrap();

        return true;
    }
    
    false
}


use serde::{Serialize, Deserialize};
use schema::*;
use std::{
    io::{prelude::*, BufReader, self},
    net::TcpStream, error::Error
};
use diesel::{
    pg::PgConnection,
    prelude::*,
};
use models::*;
use serde_json::{json, Value};

pub mod schema;
pub mod models;

pub type Eval<T> = Result<Option<T>, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Package{
    pub header: String,
    pub payload: String
}

pub fn terminate<T>()-> Eval<T>{
    Err(Box::new(io::Error::new(io::ErrorKind::Other, "Terminate")))
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

pub fn create_user(payload: NewUser)-> Eval<()>{
    let connection = &mut establish_connection();

    if users::table.filter(users::username.eq(payload.username.to_owned()))
        .first::<User>(connection).is_err(){
        diesel::insert_into(users::table)
            .values(&payload)
            .execute(connection)?;

        return Ok(Some(()));
    }
    
    Ok(None)
}

pub fn get_account_keys(payload: Value)-> Eval<String>{
    let connection = &mut establish_connection();

    if let Some(payload) = payload["username"].as_str(){
        if let Ok(user) = users::table.filter(users::username.eq(payload))
            .first::<User>(connection){
            return Ok(Some(json!({ "salt": user.salt }).to_string()));
        }
        else{
            return Ok(None);
        }
    }

   terminate::<String>()
}

pub fn validate_key(payload: Value)-> Eval<(User, bool)>{
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

    terminate::<(User, bool)>()
}

pub fn create_kanji(user: &User, mut payload: NewKanji)-> Eval<()>{
    let connection = &mut establish_connection();

    if kanji::table.filter(kanji::symbol.eq(&payload.symbol))
        .filter(kanji::user_id.eq(user.id))
        .first::<Kanji>(connection).is_err(){
        payload.user_id = user.id;

        for mut vocab in Vocab::belonging_to(&user)
            .load::<Vocab>(connection)
            .unwrap(){
            if vocab.phrase.contains(&payload.symbol){
                vocab.kanji_refs.push(Some(payload.symbol.to_owned()));

                diesel::update(vocab::table.find(vocab.id))
                    .set(vocab::kanji_refs.eq(vocab.kanji_refs))
                    .execute(connection)?;

                payload.vocab_refs.push(Some(vocab.phrase));
            }
        }

        diesel::insert_into(kanji::table)
            .values(&payload)
            .execute(connection)?;

        return Ok(Some(()));
    }
    
    Ok(None)
}

pub fn create_vocab(user: &User, mut payload: NewVocab)-> Eval<()>{
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
                    .execute(connection)?;

                payload.kanji_refs.push(Some(kanji.symbol));
           }
        }

        diesel::insert_into(vocab::table)
            .values(&payload)
            .execute(connection)?;

        return Ok(Some(()));
    }
    
    Ok(None)
}


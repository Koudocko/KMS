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
    let database_url = "postgres://postgres@localhost/SHSM_Project";

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn check_username(payload: Value)-> Result<bool, Box<dyn Error>>{
    let connection = &mut establish_connection();

    if let Some(payload) = payload["username"].as_str(){
        Ok(users::dsl::users.filter(users::dsl::username.eq(payload)).first::<User>(connection).is_err())
    }
    else{
        Err(Box::new(PlainError::new())) 
    }
}

pub fn store_in_database(new_user: NewUser)-> Result<usize, Box<dyn Error>>{
    let connection = &mut establish_connection();

    Ok(diesel::insert_into(schema::users::table)
        .values(&new_user)
        .execute(connection)?)
}

pub fn get_account_keys(payload: Value)-> Result<Option<String>, Box<dyn Error>>{
    let connection = &mut establish_connection();

    if let Some(payload) = payload["username"].as_str(){
        if let Ok(user) = users::dsl::users.filter(users::dsl::username.eq(payload)).first::<User>(connection){
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
            if let Ok(user) = users::dsl::users.filter(users::dsl::username.eq(user_username)).first::<User>(connection){
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

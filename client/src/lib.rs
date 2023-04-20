use serde::{Serialize, Deserialize};
use std::{
    io::{prelude::*, BufReader, self},
    any::Any,
};
use serde_json::{json, Value};
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::error::Error;


pub type Eval<T> = Result<T, &'static str>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Package{
    pub id: i32,
    pub header: String,
    pub payload: String
}

pub fn unpack(payload: &str, field: &str)-> Value{
    serde_json::from_str::<Value>(payload).unwrap()[field].clone()
}

pub async fn write_stream(stream: &mut TcpStream, package: Package)-> Result<(), Box<dyn Error>>{
    let mut buf = serde_json::to_vec(&package).unwrap();
    buf.push(b'\n');
    stream.write_all(&mut buf).await?;

    Ok(())
}

pub async fn read_stream(stream: &mut TcpStream)-> Result<Package, Box<dyn Error>>{
    let mut buf = [0_u8; 4096];
    if stream.read(&mut buf).await? == 0{
        return Err(Box::new(IoError::new(IoErrorKind::Other, "")));
    }

    Ok(serde_json::from_slice::<Package>(&buf[..buf.iter()
        .position(|x| *x == b'\n')
        .unwrap()
    ]).unwrap())
}

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::collections::HashMap;
use std::{
    sync::Mutex,
};
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};
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
use commands::*;
use tokio::sync::Mutex as AsyncMutex;

mod commands;

#[tokio::main]
async fn main(){
    unsafe{
        if STREAM.is_none(){
            STREAM = if let Ok(stream) = TcpStream::connect("127.0.0.1:7878").await{
                Some(AsyncMutex::new(stream))
            }
            else{
                None
            };
        }
    }
    
    tokio::spawn(async{
        unsafe{
            loop{
                if let Some(stream_ref) = &mut STREAM{
                    let mut stream_ref = stream_ref.lock().await;
                    let mut buf = [0_u8; 4096];
                     if let Ok(bytes) = stream_ref.read(&mut buf).await{
                        if bytes == 0{
                            STREAM = None;
                        }
                    }
                    else{
                        STREAM = None;
                    }

                    let package = serde_json::from_slice::<Package>(&buf[..buf.iter()
                        .position(|x| *x == b'\n')
                        .unwrap()
                    ]).unwrap();

                    PACKAGES.lock().unwrap().1.insert(package.id, package);
                }
            }
        }
    });

    // add_user("Joe biden".to_owned(), ("__joebidengaming64___".to_owned(), "__joebidengaming64___".to_owned()));
    login_user("Joe biden".to_owned(), "__joebidengaming64___".to_owned()).await;
    // loop{
    //     login_user("Joe biden".to_owned(), "__joebidengaming64___".to_owned());
    // }
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
    // tauri::Builder::default()
    //     .setup(|app|{
    //         tokio::spawn(async{
    //             unsafe{
    //                 if STREAM.is_none(){
    //                     STREAM = if let Ok(stream) = TcpStream::connect("127.0.0.1:7878").await{
    //                         Some(AsyncMutex::new(stream))
    //                     }
    //                     else{
    //                         None
    //                     };
    //                 }

    //                 loop{
    //                     if let Some(stream_ref) = &mut STREAM{
    //                         let stream_ref = stream_ref.lock().await;
    //                         let mut buf = [0_u8; 4096];
    //                          if let Ok(bytes) = stream_ref.read(&mut buf).await{
    //                             if bytes == 0{
    //                                 STREAM = None;
    //                             }
    //                         }
    //                         else{
    //                             STREAM = None;
    //                         }

    //                         let package = serde_json::from_slice::<Package>(&buf[..buf.iter()
    //                             .position(|x| *x == b'\n')
    //                             .unwrap()
    //                         ]).unwrap();

    //                         PACKAGES.lock().unwrap().1.insert(package.id, package);
    //                     }
    //                 }
    //             }
    //         });

    //         Ok(())
    //     })
    //     .invoke_handler(tauri::generate_handler![])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");
}

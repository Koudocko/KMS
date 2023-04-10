use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::schema::*;

#[derive(Identifiable, Queryable, Clone, Default)]
#[diesel(table_name = users)]
pub struct User{
    pub id: i32,
    pub username: String,
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
}

impl Hash for User{
    fn hash<H: Hasher>(&self, state: &mut H){
        self.id.hash(state);
    }
}

impl PartialEq for User{
    fn eq(&self, other: &User)-> bool{
        self.id == other.id
    }
}
impl Eq for User{}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser{
    pub username: String,
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
}

#[derive(Identifiable, Queryable, Associations)]
#[diesel(table_name = groups, belongs_to(User))]
pub struct Group{
    pub id: i32,
    pub title: String,
    pub colour: Option<String>,
    pub vocab: bool,
    pub user_id: i32,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = groups)]
pub struct NewGroup{
    pub title: String,
    pub colour: Option<String>,
    pub vocab: bool,
    pub user_id: i32,
}
#[derive(Identifiable, Queryable, Associations)]
#[diesel(table_name = kanji, belongs_to(User), belongs_to(Group))]
pub struct Kanji{
    pub id: i32,
    pub symbol: String,
    pub meaning: String,
    pub onyomi: Vec<Option<String>>,
    pub kunyomi: Vec<Option<String>>,
    pub description: Option<String>,
    pub vocab_refs: Vec<Option<String>>,
    pub user_id: i32,
    pub group_id: Option<i32>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = kanji)]
pub struct NewKanji{
    pub symbol: String,
    pub meaning: String,
    pub onyomi: Vec<Option<String>>,
    pub kunyomi: Vec<Option<String>>,
    pub description: Option<String>,
    pub vocab_refs: Vec<Option<String>>,
    pub user_id: i32,
    pub group_id: Option<i32>,
}

#[derive(Identifiable, Queryable, Associations)]
#[diesel(table_name = vocab, belongs_to(User), belongs_to(Group))]
pub struct Vocab{
    pub id: i32,
    pub phrase: String,
    pub meaning: String,
    pub reading: Vec<Option<String>>,
    pub description: Option<String>,
    pub kanji_refs: Vec<Option<String>>,
    pub user_id: i32,
    pub group_id: Option<i32>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = vocab)]
pub struct NewVocab{
    pub phrase: String,
    pub meaning: String,
    pub reading: Vec<Option<String>>,
    pub description: Option<String>,
    pub kanji_refs: Vec<Option<String>>,
    pub user_id: i32,
    pub group_id: Option<i32>,
}

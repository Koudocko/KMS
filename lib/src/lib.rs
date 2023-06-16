use serde::{Serialize, Deserialize};

pub mod schema;
pub mod models;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package{
    pub id: u8,
    pub header: String,
    pub payload: String
}

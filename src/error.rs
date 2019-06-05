use std::result;
//use rocket_contrib::uuid;
//use diesel;

pub type Result<T> = result::Result<T, Error>;

#[derive(Responder, Debug)]
pub enum Error {
    ParseId(String),
    Database(String)
}

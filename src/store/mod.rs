mod db;

use chrono::NaiveDateTime;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{fairing, fairing::Fairing, Rocket};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Event {
    pub title: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
}

type Duration = u32;

impl Occurrence {
    pub fn end(&self) -> NaiveDateTime {
        use std::convert::TryInto;
        use std::ops::Add;
        self.start
            .add(chrono::Duration::minutes(self.duration.try_into().unwrap()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Location {
    pub name: String,
    pub address: String,
}

pub struct Store(db::Connection);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }
}

pub struct StoreFairing;

impl Fairing for StoreFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Events Store Fairing",
            kind: fairing::Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        db::Connection::fairing()
            .on_attach(rocket)
            .and_then(db::initialize)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}

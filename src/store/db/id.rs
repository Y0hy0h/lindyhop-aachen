use std::fmt::Debug;
use std::io::Write;
use std::marker::PhantomData;

use diesel::backend::Backend;
use diesel::deserialize;
use diesel::expression::{bound::Bound, AsExpression};
use diesel::serialize::{self, Output};
use diesel::sql_types::{Binary, HasSqlType};
use diesel::sqlite::Sqlite;
use diesel::types::{FromSql, ToSql};
use serde::Deserialize;
use uuid::Uuid;

use crate::store;

// SqlId implementation inspired by https://github.com/forte-music/core/blob/fc9cd6217708b0dd6ae684df3a53276804479c59/src/models/id.rs#L67
#[derive(Debug, Deserialize, FromSqlRow, Clone, Hash, PartialEq, Eq)]
pub struct SqlId<Item>(Uuid, PhantomData<Item>);

impl<Item> SqlId<Item> {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl<Item> From<Uuid> for SqlId<Item> {
    fn from(uuid: Uuid) -> Self {
        SqlId(uuid, PhantomData)
    }
}

impl<Item> From<SqlId<Item>> for store::Id<Item> {
    fn from(id: SqlId<Item>) -> store::Id<Item> {
        id.0.into()
    }
}

impl<Item> From<store::Id<Item>> for SqlId<Item> {
    fn from(id: store::Id<Item>) -> SqlId<Item> {
        id.id.into()
    }
}

impl<DB: Backend + HasSqlType<Binary>, Item: Debug> ToSql<Binary, DB> for SqlId<Item> {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        let bytes = self.0.as_bytes();
        <[u8] as ToSql<Binary, DB>>::to_sql(bytes, out)
    }
}

impl<Item> FromSql<Binary, Sqlite> for SqlId<Item> {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let bytes_vec = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        Ok(Uuid::from_slice(&bytes_vec)?.into())
    }
}

impl<Item> AsExpression<Binary> for SqlId<Item> {
    type Expression = Bound<Binary, SqlId<Item>>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a, Item> AsExpression<Binary> for &'a SqlId<Item> {
    type Expression = Bound<Binary, &'a SqlId<Item>>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

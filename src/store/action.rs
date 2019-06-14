use std::collections::HashMap;

use diesel::QueryResult;

pub trait Actions<T> {
    type Id;
    type DeleteError;

    fn all(&self) -> HashMap<Self::Id, T>;
    fn create(&self, item: T) -> QueryResult<Self::Id>;
    fn read(&self, id: Self::Id) -> QueryResult<T>;
    fn update(&self, id: Self::Id, new_item: T) -> QueryResult<T>;
    fn delete(&self, id: Self::Id) -> Result<T, Self::DeleteError>;
}

macro_rules! derive_actions {
    ($t: ident, $s: ident) => {
        impl Actions<$t> for Store {
            type Id = Id;
            type DeleteError = diesel::result::Error;

            fn all(&self) -> HashMap<Self::Id, $t> {
                schema
                    .load::<$s>(&*self.0)
                    .expect("Could not load database")
                    .into_iter()
                    .map(|x| x.into())
                    .collect()
            }

            fn create(&self, item: $t) -> QueryResult<Self::Id> {
                let sql_item: $s = item.into();
                diesel::insert_into(table)
                    .values(&sql_item)
                    .execute(&*self.0)?;

                Ok(sql_item.id.into())
            }

            fn read(&self, item_id: Self::Id) -> QueryResult<$t> {
                use super::db::SqlId;

                schema
                    .find(SqlId::from(item_id))
                    .first::<$s>(&*self.0)
                    .map(|x| x.into())
                    .map(|(_, x)| x)
            }

            fn update(&self, item_id: Self::Id, new_item: $t) -> QueryResult<$t> {
                use super::db::SqlId;

                let raw_id: SqlId = item_id.into();
                let (_, previous): (Id, $t) = schema.find(&raw_id).first::<$s>(&*self.0)?.into();

                diesel::update(schema.find(&raw_id))
                    .set::<$s>(new_item.into())
                    .execute(&*self.0)?;

                Ok(previous)
            }

            fn delete(&self, id: Self::Id) -> Result<$t, Self::DeleteError> {
                use db::SqlId;
                let raw_id: SqlId = id.into();
                let (_, previous): (Id, $t) = schema.find(&raw_id).first::<$s>(&*self.0)?.into();

                diesel::delete(schema.find(&raw_id)).execute(&*self.0)?;

                Ok(previous)
            }
        }
    };
}

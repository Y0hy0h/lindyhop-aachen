mod db;

use db::{SqlId, SqlLocation};

#[juniper::object]
impl SqlLocation {
    fn id(&self) -> &SqlId {
        &self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn address(&self) -> &String {
        &self.address
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn new_schema_instance() -> Schema {
    Schema::new(Query, Mutation::new())
}

struct Query;

#[juniper::object(Context=Store)]
impl Query {
    fn locations(context: &Store) -> Vec<SqlLocation> {
        use db::schema::locations::dsl::*;
        use diesel::RunQueryDsl;
        locations
            .load(&*context.0)
            .expect("Error loading from database.")
    }
}

type Mutation = juniper::EmptyMutation<Store>;

pub struct Store(db::Connection);

impl juniper::Context for Store {}

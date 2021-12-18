use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::State;

use std::convert::Into;
use std::ops::Deref;

pub mod models;
pub mod schema;

pub type DbConnPool = Pool<ConnectionManager<SqliteConnection>>;
type DbPooledConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub struct Db(DbPooledConn);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Db {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        request
            .guard::<&State<DbConnPool>>()
            .await
            .and_then(|pool| match pool.get() {
                Ok(db_conn) => request::Outcome::Success(Db(db_conn)),
                _ => request::Outcome::Failure((Status::InternalServerError, ())),
            })
    }
}

impl Deref for Db {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<Db> for DbPooledConn {
    fn into(self) -> Db {
        Db(self)
    }
}

pub fn init_pool() -> DbConnPool {
    let manager = ConnectionManager::<SqliteConnection>::new(
        dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env variable"),
    );

    Pool::builder()
        .build(manager)
        .expect("failed to create DB connection pool")
}

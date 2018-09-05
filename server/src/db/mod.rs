use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

use dotenv;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};

use std::ops::Deref;


pub mod models;
pub mod schema;


pub type DbConnPool = Pool<ConnectionManager<SqliteConnection>>;
type DbPooledConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub struct Db(DbPooledConn);

impl<'a, 'r> FromRequest<'a, 'r> for Db {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<DbConnPool>>()?;

        match pool.get() {
            Ok(conn) => Outcome::Success(Db(conn)),
            Err(_) => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}

impl Deref for Db {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
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

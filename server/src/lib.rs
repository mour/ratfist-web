#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use diesel::sqlite::SqliteConnection;

#[cfg(feature = "meteo")]
pub mod meteo;

pub mod comm;
pub mod db;
mod utils;

#[derive(Debug)]
pub struct CoreError;

embed_migrations!("migrations");

pub fn run_migrations(connection: &SqliteConnection) {
    embedded_migrations::run(connection).expect("Error while running DB migrations.");
}

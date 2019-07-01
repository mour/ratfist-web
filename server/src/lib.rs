#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;

#[cfg(feature = "spinner")]
pub mod spinner;

#[cfg(feature = "meteo")]
pub mod meteo;

pub mod comm;
pub mod db;
mod utils;

#[derive(Debug)]
pub struct CoreError;

#![feature(plugin, custom_derive)]
#![feature(inclusive_range_syntax)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

extern crate regex;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;

extern crate serial;

#[cfg(feature = "spinner")]
mod spinner;


mod comm;
mod utils;


fn main() {
    let (comm, _join_handle) = comm::init();
    let rocket = rocket::ignite().manage(comm);

    #[cfg(feature = "spinner")]
    let rocket = rocket.mount("/spinner", spinner::get_routes());

    rocket.launch();
}

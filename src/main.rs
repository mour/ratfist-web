#![feature(plugin, custom_derive)]
#![feature(drop_types_in_const)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;

extern crate serial;

#[cfg(feature = "spinner")]
mod spinner;

mod comm;


fn main() {
    let _join_handle = comm::init();
    let rocket = rocket::ignite();

    #[cfg(feature = "spinner")]
    let rocket = rocket.mount("/spinner", spinner::get_routes());

    rocket.launch();
}

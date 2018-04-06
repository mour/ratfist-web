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

extern crate dotenv;

extern crate chrono;

#[cfg(feature = "spinner")]
mod spinner;

#[cfg(feature = "meteo")]
mod meteo;

mod comm;
mod utils;

fn main() {
    let path = dotenv::dotenv().ok();

    let rocket = rocket::ignite();

    // Rocket initialized above to enable logging.
    debug!("Loaded .env from: {:?}.", path);
    if log::max_log_level() >= log::LogLevelFilter::Trace {
        trace!("Loaded environment variables:");
        for (var, value) in dotenv::vars() {
            trace!("{} - {}", var, value);
        }
    }

    let (comm, _join_handle) = comm::init();
    let rocket = rocket.manage(comm);

    #[cfg(feature = "spinner")]
    let rocket = rocket.mount("/spinner", spinner::get_routes());

    #[cfg(feature = "meteo")]
    let rocket = rocket.mount("/meteo", meteo::get_routes());

    rocket.launch();
}

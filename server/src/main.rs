#![feature(plugin, custom_derive)]
#![feature(try_from)]
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

#[macro_use]
extern crate diesel;

extern crate scheduled_executor;

#[cfg(feature = "spinner")]
mod spinner;

#[cfg(feature = "meteo")]
mod meteo;

mod comm;
mod db;
mod utils;

use std::time::Duration;

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

    let db_pool = db::init_pool();
    let rocket = rocket.manage(db_pool.clone());

    let (comm, _join_handle) = comm::init();
    let rocket = rocket.manage(comm.clone());

    let executor =
        scheduled_executor::CoreExecutor::new().expect("Could not start periodic task executor");

    #[cfg(feature = "spinner")]
    let rocket = rocket.mount("/spinner", spinner::get_routes());

    #[cfg(feature = "meteo")]
    let rocket = {
        let db_pool_clone = db_pool.clone();
        let comm_clone = comm.clone();

        let fetcher_task_rate = dotenv::var("METEO_FETCHER_TASK_RATE_SECS")
            .expect("missing METEO_FETCHER_TASK_RATE_SECS env variable")
            .parse()
            .expect("METEO_FETCHER_TASK_RATE_SECS parsing error");

        executor.schedule_fixed_rate(
            Duration::from_secs(fetcher_task_rate),
            Duration::from_secs(fetcher_task_rate),
            move |_remote| {
                if meteo::fetcher::fetcher_iteration(&db_pool_clone, &comm_clone).is_err() {
                    warn!("Fetcher task error.");
                }
            },
        );

        rocket.mount("/meteo", meteo::get_routes())
    };

    rocket.launch();
}

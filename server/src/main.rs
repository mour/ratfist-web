#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_derive;

use dotenv;

#[macro_use]
extern crate diesel;

use scheduled_executor;

use std::time::Duration;

use log::{debug, trace, warn};

#[cfg(feature = "spinner")]
mod spinner;

#[cfg(feature = "meteo")]
mod meteo;

mod comm;
mod db;
mod utils;


#[derive(Debug)]
pub struct CoreError;


fn main() {
    let path = dotenv::dotenv().ok();

    let rocket = rocket::ignite();

    // Rocket initialized above to enable logging.
    debug!("Loaded .env from: {:?}.", path);
    if log::max_level() >= log::LevelFilter::Trace {
        trace!("Loaded environment variables:");
        for (var, value) in dotenv::vars() {
            trace!("{} - {}", var, value);
        }
    }

    let db_pool = db::init_pool();
    let rocket = rocket.manage(db_pool.clone());

    let (comm, _join_handle) = comm::init(&db_pool);
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

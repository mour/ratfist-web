
use log::{debug, trace, warn};

#[cfg(feature = "meteo")]
use std::time::Duration;

#[cfg(feature = "meteo")]
use ratfist_server::meteo;

use ratfist_server::{run_migrations, db};

#[rocket::main]
async fn main() {
    let path = dotenv::dotenv().ok();

    let rocket = rocket::build();

    // Rocket initialized above to enable logging.
    debug!("Loaded .env from: {:?}.", path);
    if log::max_level() >= log::LevelFilter::Trace {
        trace!("Loaded environment variables:");
        for (var, value) in dotenv::vars() {
            trace!("{} - {}", var, value);
        }
    }

    let db_pool = db::init_pool();
    let connection = db_pool.get().expect("Could not get DB connection.");
    run_migrations(&connection);

    let rocket = rocket.manage(db_pool.clone());

    #[cfg(feature = "meteo")]
    let executor =
        scheduled_executor::CoreExecutor::new().expect("Could not start periodic task executor");

    #[cfg(feature = "meteo")]
    let rocket = {
        let node_registry = meteo::node::SensorNodeRegistry::new(
            connection.into(),
        )
        .expect("Failed to construct node registry.");

        let fetcher_task_rate = dotenv::var("METEO_FETCHER_TASK_RATE_SECS")
            .expect("Missing METEO_FETCHER_TASK_RATE_SECS env variable")
            .parse()
            .expect("METEO_FETCHER_TASK_RATE_SECS parsing error");

        let node_registry_clone = node_registry.clone();

        executor.schedule_fixed_rate(
            Duration::from_secs(fetcher_task_rate),
            Duration::from_secs(fetcher_task_rate),
            move |_remote| {
                if meteo::fetcher::fetcher_iteration(&db_pool, &node_registry_clone).is_err() {
                    warn!("Fetcher task error.");
                }
            },
        );

        rocket
            .manage(node_registry)
            .mount("/meteo", meteo::get_routes())
    };

    rocket.launch().await.expect("Failed to launch server.");
}

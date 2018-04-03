extern crate serial;

#[macro_use(crate_version)]
extern crate clap;

extern crate ctrlc;

#[macro_use]
extern crate log;
extern crate env_logger;

mod dispatcher;
mod meteo;

use serial::prelude::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    // Setup program usage & get arguments
    let matches = clap::App::new("Ratfist MCU node stub")
        .version(crate_version!())
        .arg_from_usage("<serial port> 'device file of the serial port used for communication'")
        .get_matches();

    // Initialize logger
    env_logger::init();

    // Setup Ctrl-C handler
    let end_condition = Arc::new(AtomicBool::new(false));
    let end_condition_handler = end_condition.clone();

    ctrlc::set_handler(move || end_condition_handler.store(true, Ordering::SeqCst))
        .expect("error setting signal handler");

    // Init serial port
    let sp_path = matches
        .value_of("serial port")
        .expect("missing serial port path");

    let mut serial_port = serial::open(&sp_path).expect("could not open serial port");

    let settings = serial::PortSettings {
        baud_rate: serial::Baud115200,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };
    serial_port
        .configure(&settings)
        .expect("could not configure the serial port");

    // Start dispatcher & loop until Ctrl-C
    let mut disp = dispatcher::Dispatcher::new(serial_port);
    disp.register_handler_module("METEO", Box::new(meteo::MeteoModule));

    trace!("Starting main loop.");

    disp.run_until(&end_condition);

    trace!("Graceful end.");
}

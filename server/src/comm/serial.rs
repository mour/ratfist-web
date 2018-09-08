
use dotenv;

use serial;
use serial::prelude::*;

use std::thread;

use std::sync::mpsc;

use super::CommChannelTx;


pub fn create_serial_comm_task(serial_id: u32) -> (CommChannelTx, thread::JoinHandle<()>) {
    let env_var_str = format!("SERIAL_PORT_{}_PATH", serial_id);

    let mut serial_port = serial::open(
        &dotenv::var(&env_var_str).expect(&format!("missing {} env variable", env_var_str)),
    ).expect("could not open serial port");

    let settings = serial::PortSettings {
        baud_rate: serial::Baud115200,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };
    serial_port
        .configure(&settings)
        .expect("Could not configure the serial port.");

    let (channel_tx, channel_rx) = mpsc::channel();

    let join_handle = thread::spawn(|| super::comm_func(channel_rx, serial_port));

    (CommChannelTx(channel_tx), join_handle)
}
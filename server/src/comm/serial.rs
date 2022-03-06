use std::collections::HashMap;

use serial::prelude::*;

use std::thread;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use std::io::{Read, Write};

use std::time::Duration;

use log::{debug, warn};

use crate::utils::Result;
use anyhow::anyhow;

type MsgAndResponseChannel = (u32, String, Sender<String>);

#[derive(Clone)]
pub struct CommChannelTx(Sender<MsgAndResponseChannel>);

impl CommChannelTx {
    pub fn send(&self, node_id: u32, msg: String) -> Result<String> {
        let (response_tx, response_rx) = mpsc::channel();

        self.0
            .send((node_id, msg, response_tx))
            .map_err(|e| anyhow!("Failed to send message. {e:?}"))?;

        let raw_response_msg = response_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|e| anyhow::anyhow!("Error while receiving serial message. {e:?}"))?;

        Ok(raw_response_msg)
    }
}

fn calc_checksum(input: &str) -> u8 {
    input.as_bytes().iter().fold(0, |csum, ch| csum ^ ch)
}

fn process_incoming_msg(raw_msg: &str) -> Result<(u64, &str)> {
    if raw_msg.len() < 4 {
        return Err(anyhow!("Incoming message too short.").into());
    }

    if raw_msg.as_bytes()[raw_msg.len() - 3] as char != '*' {
        return Err(anyhow!("Incoming message invalid format.").into());
    }

    let packet_csum = u8::from_str_radix(&raw_msg[(raw_msg.len() - 2)..], 16).unwrap();

    let msg_str = &raw_msg[..(raw_msg.len() - 3)];
    let calc_csum = calc_checksum(msg_str);

    if packet_csum != calc_csum {
        return Err(anyhow!("Incoming message invalid checksum. Expecting 0x{calc_csum:2X}, got 0x{packet_csum:2X}.").into());
    }

    let comma_pos = msg_str
        .find(',')
        .ok_or(anyhow!("Incoming message invalid format."))?;
    let transaction_id = msg_str[..comma_pos]
        .parse()
        .map_err(|e| anyhow!("Incoming message parsing error. {e:?}"))?;
    let msg_payload_str = &msg_str[(comma_pos + 1)..];

    Ok((transaction_id, msg_payload_str))
}

fn comm_func<T>(channel_rx: Receiver<MsgAndResponseChannel>, mut comm: T) -> !
where
    T: Read + Write,
{
    let mut transaction_id_ctr: u64 = 0;
    let mut pending_transactions = HashMap::new();

    let mut current_packet = String::new();

    enum ParserState {
        WaitingForDollar,
        Receiving,
    }

    let mut current_state = ParserState::WaitingForDollar;

    loop {
        // Transmit all pending messages
        while let Ok((node_id, msg, resp_tx)) = channel_rx.try_recv() {
            transaction_id_ctr = transaction_id_ctr.wrapping_add(1);

            pending_transactions.insert(transaction_id_ctr, resp_tx);

            let mut out = format!("${},{},{}*", transaction_id_ctr, node_id, msg);
            let csum = calc_checksum(&out[1..(out.len() - 1)]);
            out.push_str(&format!("{:02X}\r\n", csum));

            debug!("server -> mcu: '{}'", &out[..(out.len() - 2)]);

            let _ = comm.write_all(out.as_bytes());
        }

        // Parse all incoming chars
        let mut incoming = [0; 100];
        while let Ok(incoming_len) = comm.read(&mut incoming) {
            debug!("Rx buffer is now: {:?}", incoming.to_vec());

            for byte in incoming.iter().take(incoming_len) {
                let ch = char::from(*byte);

                match current_state {
                    ParserState::WaitingForDollar => {
                        if ch == '$' {
                            current_state = ParserState::Receiving;
                        }
                    }
                    ParserState::Receiving => {
                        current_packet.push(ch);

                        let len = current_packet.len();
                        if len >= 2 && &current_packet[(len - 2)..] == "\r\n" {
                            match process_incoming_msg(&current_packet[..(len - 2)]) {
                                Ok((trans_id, payload_str)) => {
                                    debug!(
                                        "Received trans id {}, payload {}",
                                        trans_id, payload_str
                                    );

                                    if let Some(response_channel) =
                                        pending_transactions.remove(&trans_id)
                                    {
                                        let _ = response_channel.send(payload_str.to_string());
                                    } else {
                                        warn!("Unexpected transition id {}!", trans_id);
                                    }
                                }
                                Err(_) => {
                                    warn!("Unexpected response: '{}'", &current_packet[..(len - 2)])
                                }
                            }

                            current_packet.clear();
                            current_state = ParserState::WaitingForDollar;
                        }
                    }
                }
            }
        }
    }
}

pub fn create_serial_comm_task(serial_id: u32) -> Result<(CommChannelTx, thread::JoinHandle<()>)> {
    let env_var_str = format!("SERIAL_PORT_{}_PATH", serial_id);

    let mut serial_port = serial::open(
        &dotenv::var(&env_var_str)
            .map_err(|e| anyhow!("Missing {env_var_str} env variable. {e:?}"))?,
    )
    .map_err(|e| anyhow!("Could not open serial port. {e:?}"))?;

    let settings = serial::PortSettings {
        baud_rate: serial::Baud115200,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };
    serial_port
        .configure(&settings)
        .map_err(|e| anyhow!("Could not configure the serial port. {e:?}"))?;

    let (channel_tx, channel_rx) = mpsc::channel();

    let join_handle = thread::spawn(|| comm_func(channel_rx, serial_port));

    Ok((CommChannelTx(channel_tx), join_handle))
}

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SendError, Sender};

use std::io::{Read, Write};

use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use std::thread;

use db::models::Node;
use db::DbConnPool;

use diesel::prelude::*;

use CoreError;

mod serial;

type MsgAndResponseChannel = (String, Sender<String>);

#[derive(Clone)]
pub struct CommState(Arc<HashMap<usize, Mutex<CommChannelTx>>>);

impl CommState {
    pub fn get_comm_channel(&self, node_id: usize) -> Result<CommChannelTx, CoreError> {
        let comm = self
            .0
            .get(&node_id)
            .ok_or(CoreError)?
            .lock()
            .expect("mutex poisoned");

        Ok(comm.clone())
    }
}

#[derive(Clone)]
pub struct CommChannelTx(Sender<MsgAndResponseChannel>);

impl CommChannelTx {
    pub fn send(&self, msg: String) -> Result<Receiver<String>, SendError<MsgAndResponseChannel>> {
        let (response_tx, response_rx) = mpsc::channel();

        self.0.send((msg, response_tx))?;

        Ok(response_rx)
    }
}

fn calc_checksum(input: &str) -> u8 {
    input.as_bytes().into_iter().fold(0, |csum, ch| csum ^ ch)
}

fn process_incoming_msg(raw_msg: &str) -> Result<(u64, &str), ()> {
    if raw_msg.len() < 4 {
        return Err(());
    }

    if raw_msg.as_bytes()[raw_msg.len() - 3] as char != '*' {
        return Err(());
    }

    let packet_csum = u8::from_str_radix(&raw_msg[(raw_msg.len() - 2)..], 16).unwrap();

    let msg_str = &raw_msg[..(raw_msg.len() - 3)];
    let calc_csum = calc_checksum(msg_str);

    if packet_csum != calc_csum {
        return Err(());
    }

    let comma_pos = msg_str.find(',').ok_or(())?;
    let transaction_id = msg_str[..comma_pos].parse().map_err(|_| ())?;
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
        while let Ok((msg, resp_tx)) = channel_rx.try_recv() {
            transaction_id_ctr = transaction_id_ctr.wrapping_add(1);

            pending_transactions.insert(transaction_id_ctr, resp_tx);

            let mut out = format!("${},{}*", transaction_id_ctr, msg);
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

pub fn init(db_conn_pool: &DbConnPool) -> (CommState, Vec<thread::JoinHandle<()>>) {
    // Get list of nodes, and their appropriate comm routing info
    let db = db_conn_pool.get().expect("could not get DB connection");

    let nodes = {
        use db::schema::*;

        nodes::table.load::<Node>(&db)
    }.expect("could not load node info from DB");


    // Open the appropriate listeners
    let mut node_channels = HashMap::new();
    let mut join_handles = Vec::new();

    for node in nodes {
        match node.route_type.as_str() {
            "serial" => {
                let serial_route_id = node.route_param
                    .expect("missing serial route ID in DB")
                    .parse()
                    .expect("failed to parse serial route ID");

                node_channels.entry(serial_route_id).or_insert_with(|| {
                    let (channel_tx, join_handle) = serial::create_serial_comm_task(serial_route_id);

                    join_handles.push(join_handle);

                    Mutex::new(channel_tx)
                });
            }
            unknown_type_str => panic!("unknown comm route type: {}", unknown_type_str),
        }
    }

    (CommState(Arc::new(node_channels)), join_handles)
}

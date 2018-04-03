use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::collections::HashMap;

use std::ops::{Deref, DerefMut};

pub trait Module {
    fn handle_incoming_msg(
        &mut self,
        msg_writer: &mut MsgSender,
        transaction_id: u32,
        msg_str: &str,
    ) -> Result<(), ()>;
}

pub trait MsgSender {
    fn write_msg(
        &mut self,
        transaction_id: u32,
        module_name: &str,
        msg_str: &str,
    ) -> Result<(), ()>;
}

enum ParserState {
    WaitingForDollar,
    Receiving,
}

struct Comm<T: Write + Read>(T);

impl<T> Deref for Comm<T>
where
    T: Write + Read,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Comm<T>
where
    T: Write + Read,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct Dispatcher<T: Write + Read> {
    comm: Comm<T>,
    modules: HashMap<String, Box<Module>>,
}

impl<T> Dispatcher<T>
where
    T: Write + Read,
{
    pub fn new(comm: T) -> Self {
        Dispatcher {
            comm: Comm(comm),
            modules: HashMap::new(),
        }
    }

    pub fn register_handler_module(&mut self, module_name: &str, module: Box<Module>) -> bool {
        self.modules.insert(module_name.into(), module).is_some()
    }

    pub fn run_until(&mut self, done_flag: &Arc<AtomicBool>) {
        let mut curr_msg = String::new();
        let mut parser_state = ParserState::WaitingForDollar;

        while !done_flag.load(Ordering::SeqCst) {
            let mut recv_buf = [0; 100];

            if let Ok(incoming_len) = self.comm.read(&mut recv_buf) {
                for byte in recv_buf.into_iter().take(incoming_len) {
                    let ch = *byte as char;

                    match parser_state {
                        ParserState::WaitingForDollar => {
                            if ch == '$' {
                                parser_state = ParserState::Receiving;
                            }
                        }
                        ParserState::Receiving => {
                            curr_msg.push(ch);

                            let len = curr_msg.len();

                            if len >= 2 && &curr_msg[(len - 2)..] == "\r\n" {
                                self.handle_incoming_msg(&curr_msg[..(len - 2)]);

                                curr_msg.clear();
                                parser_state = ParserState::WaitingForDollar;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_incoming_msg(&mut self, raw_msg: &str) {
        trace!("Incoming raw message: {}", raw_msg);

        if raw_msg.len() < 4 || raw_msg.as_bytes()[raw_msg.len() - 3] as char != '*' {
            warn!("Invalid message: {}", raw_msg);
            return;
        }

        let msg_csum = match u8::from_str_radix(&raw_msg[(raw_msg.len() - 2)..], 16) {
            Ok(val) => val,
            Err(_) => {
                warn!("Could not parse message checksum: {}", raw_msg);
                return;
            }
        };

        let msg_str = &raw_msg[..(raw_msg.len() - 3)];
        let expected_csum = calc_csum(msg_str);

        if msg_csum != expected_csum {
            warn!("Invalid checksum in message: {}", raw_msg);
            warn!(
                "Expected '{:02X}', but received '{:02X}'.",
                expected_csum, msg_csum
            );
            return;
        }

        let mut msg_parts = msg_str.splitn(3, ',');

        let transaction_id = match msg_parts
            .next()
            .ok_or(())
            .and_then(|val| val.parse().map_err(|_| ()))
        {
            Ok(val) => val,
            Err(_) => {
                warn!("Could not parse transaction ID from message: {}", msg_str);
                return;
            }
        };

        let module_name = match msg_parts.next() {
            Some(val) => val,
            None => {
                warn!("Could not parse module name from message: {}", msg_str);
                return;
            }
        };

        let msg_payload_str = match msg_parts.next() {
            Some(val) => val,
            None => {
                warn!("Message does not contain payload: {}", msg_str);
                return;
            }
        };

        if let Some(handler) = self.modules.get_mut(module_name) {
            if handler
                .handle_incoming_msg(&mut self.comm, transaction_id, msg_payload_str)
                .is_err()
            {
                warn!("Error while handling message: {}", msg_str);
            }
        } else {
            warn!("Unknown module name in message: {}", msg_str);
            warn!("Module name: {}", module_name);
        }
    }
}

impl<T> MsgSender for Comm<T>
where
    T: Write + Read,
{
    fn write_msg(
        &mut self,
        transaction_id: u32,
        module_name: &str,
        msg_str: &str,
    ) -> Result<(), ()> {
        let mut out_str = format!("${},{},{}", transaction_id, module_name, msg_str);

        let csum = calc_csum(&out_str[1..]);

        out_str.push_str(&format!("*{:02X}\r\n", csum));

        trace!(
            "Responding with message: {}",
            &out_str[..(out_str.len() - 2)]
        );

        self.0
            .write(out_str.as_bytes())
            .map_err(|_| ())
            .and_then(|bytes_written| {
                if bytes_written == out_str.as_bytes().len() {
                    Ok(())
                } else {
                    Err(())
                }
            })
    }
}

fn calc_csum(msg_str: &str) -> u8 {
    msg_str.as_bytes().into_iter().fold(0, |csum, ch| csum ^ ch)
}

use dispatcher::Module;
use dispatcher::MsgSender;

pub struct MeteoModule;

impl Module for MeteoModule {
    fn handle_incoming_msg(
        &mut self,
        msg_writer: &mut MsgSender,
        transaction_id: u32,
        msg_str: &str,
    ) -> Result<(), ()> {
        trace!("Handling message: {} {}", transaction_id, msg_str);

        let mut values = msg_str.split(',');

        let msg_type = values.next().ok_or(())?;
        let ch_num = values.next().ok_or(())?.parse::<u32>().map_err(|_| ())?;

        let response_payload_str = match msg_type {
            "GET_TEMPERATURE" => format!("TEMPERATURE_REPLY,{},0", ch_num),
            "GET_HUMIDITY" => format!("HUMIDITY_REPLY,{},0", ch_num),
            "GET_PRESSURE" => format!("PRESSURE_REPLY,{},0", ch_num),
            "GET_LIGHT_LEVEL" => format!("LIGHT_LEVEL_REPLY,{},0", ch_num),
            unknown_msg_type_str => {
                warn!("Unknown message type: {}", unknown_msg_type_str);
                return Err(());
            }
        };

        msg_writer.write_msg(transaction_id, "METEO", &response_payload_str)
    }
}

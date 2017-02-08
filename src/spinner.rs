#![allow(unmounted_route)]

use rocket::Route;

use rocket::Response;
use rocket::http::Status;

use rocket_contrib::JSON;

use std::str::FromStr;

use std::collections::HashMap;

use std::time::Duration;

use comm;

#[derive(Debug)]
struct SpinnerError;

pub enum OutgoingMessage {
    SetState(u64, ChannelCommand),
    GetState(u64),
    SetPlan(u64, Vec<PlanLeg>),
    GetPlan(u64),
}

impl From<OutgoingMessage> for String {
    fn from(msg: OutgoingMessage) -> String {
        match msg {
            OutgoingMessage::SetState(channel_id, ref cmd) => {
                format!("SET_SPIN_STATE,{},{}",
                        channel_id,
                        match *cmd {
                            ChannelCommand::Start => "START",
                            ChannelCommand::Stop => "STOP",
                        })
            }
            OutgoingMessage::GetState(channel_id) => format!("GET_SPIN_STATE,{}", channel_id),
            OutgoingMessage::SetPlan(channel_id, ref plan) => {
                let mut msg = format!("SET_PLAN,{}", channel_id);
                for leg in plan {
                    msg = format!("{},{},{}", msg, leg.target_val_pct, leg.duration_msecs);
                }

                msg
            }
            OutgoingMessage::GetPlan(channel_id) => format!("GET_PLAN,{}", channel_id),
        }
    }
}


enum IncomingMessage {
    Plan(u64, Vec<PlanLeg>),
    State(u64, ChannelState),
    Ret(i64),
}

impl FromStr for IncomingMessage {
    type Err = SpinnerError;

    fn from_str(s: &str) -> Result<IncomingMessage, Self::Err> {
        let mut tokens = s.split(',');
        if let Some(msg_type) = tokens.next() {
            match msg_type {
                "SPIN_PLAN_REPLY" => {
                    let ch_num =
                        tokens.next().ok_or(SpinnerError)?.parse().map_err(|_| SpinnerError)?;

                    let mut plan_legs = Vec::new();

                    let mut plan_tokens = tokens.peekable();

                    while plan_tokens.peek().is_some() {
                        let duration_msecs = plan_tokens.next()
                            .ok_or(SpinnerError)?
                            .parse()
                            .map_err(|_| SpinnerError)?;
                        let target_val_pct = plan_tokens.next()
                            .ok_or(SpinnerError)?
                            .parse()
                            .map_err(|_| SpinnerError)?;

                        plan_legs.push(PlanLeg {
                            target_val_pct: target_val_pct,
                            duration_msecs: duration_msecs,
                        });
                    }

                    Ok(IncomingMessage::Plan(ch_num, plan_legs))
                }
                "SPIN_STATE_REPLY" => {
                    let ch_num =
                        tokens.next().ok_or(SpinnerError)?.parse().map_err(|_| SpinnerError)?;

                    let state_str = tokens.next().ok_or(SpinnerError)?;

                    let pos_in_plan_msecs =
                        tokens.next().ok_or(SpinnerError)?.parse().map_err(|_| SpinnerError)?;
                    let output_val_pct =
                        tokens.next().ok_or(SpinnerError)?.parse().map_err(|_| SpinnerError)?;

                    match state_str {
                        "RUNNING" => {
                            Ok(IncomingMessage::State(ch_num,
                                                      ChannelState::Running {
                                                          output_val_pct: output_val_pct,
                                                          pos_in_plan_msecs: pos_in_plan_msecs,
                                                      }))
                        }
                        "STOPPED" => Ok(IncomingMessage::State(ch_num, ChannelState::Stopped)),
                        _ => Err(SpinnerError),
                    }
                }
                "RET" => {
                    let ret_val =
                        tokens.next().ok_or(SpinnerError)?.parse().map_err(|_| SpinnerError)?;
                    Ok(IncomingMessage::Ret(ret_val))
                }
                _ => Err(SpinnerError),
            }
        } else {
            Err(SpinnerError)
        }
    }
}


#[derive(Serialize, Clone)]
pub enum ChannelState {
    Running {
        output_val_pct: f64,
        pos_in_plan_msecs: u64,
    },
    Stopped,
}

#[derive(Deserialize)]
pub enum ChannelCommand {
    Start,
    Stop,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlanLeg {
    target_val_pct: f64,
    duration_msecs: u64,
}



const NUM_CHANNELS: u64 = 1;



fn send_msg(msg_str: String, comm: &comm::CommChannelTx) -> Result<String, SpinnerError> {
    debug!("Sending: {}", msg_str);

    let response_channel = comm.send(msg_str)
        .map_err(|e| {
            warn!("{}", e);
            SpinnerError
        })?;

    let raw_response_msg = response_channel.recv_timeout(Duration::from_secs(3))
        .map_err(|e| {
            warn!("{}", e);
            SpinnerError
        })?;

    debug!("Response: {}", raw_response_msg);

    Ok(raw_response_msg)
}


fn send_channel_state_query_msg(id: u64,
                                comm: &comm::CommChannelTx)
                                -> Result<ChannelState, SpinnerError> {

    let response_str = send_msg(OutgoingMessage::GetPlan(id).into(), comm)?;

    match response_str.parse()? {
        IncomingMessage::State(ret_id, ref state) if ret_id == id => Ok(state.clone()),
        _ => Err(SpinnerError),
    }
}

fn send_channel_plan_query_msg(id: u64,
                               comm: &comm::CommChannelTx)
                               -> Result<Vec<PlanLeg>, SpinnerError> {

    let response_str = send_msg(OutgoingMessage::GetState(id).into(), comm)?;

    match response_str.parse()? {
        IncomingMessage::Plan(ret_id, ref plan) if ret_id == id => Ok(plan.clone()),
        _ => Err(SpinnerError),
    }
}




#[get("/channels")]
fn query_channels(comm: comm::CommChannelTx)
                  -> Result<JSON<HashMap<u64, (ChannelState, Vec<PlanLeg>)>>, SpinnerError> {
    let mut map = HashMap::new();

    for ch_num in 0..NUM_CHANNELS {
        map.insert(ch_num,
                   (send_channel_state_query_msg(ch_num, &comm)?,
                    send_channel_plan_query_msg(ch_num, &comm)?));
    }

    Ok(JSON(map))
}

#[get("/channels/<id>")]
fn query_channel(id: u64,
                 comm: comm::CommChannelTx)
                 -> Result<Option<JSON<(ChannelState, Vec<PlanLeg>)>>, SpinnerError> {


    Ok(Some(JSON((send_channel_state_query_msg(id, &comm)?,
                  send_channel_plan_query_msg(id, &comm)?))))
}


#[get("/channels/<id>/state")]
fn query_channel_state(id: u64,
                       comm: comm::CommChannelTx)
                       -> Result<Option<JSON<ChannelState>>, SpinnerError> {

    match send_channel_state_query_msg(id, &comm) {
        Ok(state) => Ok(Some(JSON(state))),
        Err(_) => Err(SpinnerError),
    }
}

#[post("/channels/<id>/state", format = "application/json", data = "<new_state>")]
fn set_channel_state<'a>(id: u64,
                         new_state: JSON<ChannelCommand>,
                         comm: comm::CommChannelTx)
                         -> Result<Response<'a>, SpinnerError> {

    let response_str = send_msg(OutgoingMessage::SetState(id, new_state.into_inner()).into(),
                                &comm)?;

    match response_str.parse() {
        Ok(IncomingMessage::Ret(ret_val)) => {
            if ret_val == 0 {
                Ok(Response::build().status(Status::Accepted).finalize())
            } else {
                Ok(Response::build().status(Status::Conflict).finalize())
            }
        }
        _ => Err(SpinnerError),
    }
}

#[get("/channels/<id>/plan")]
fn query_plan(id: u64,
              comm: comm::CommChannelTx)
              -> Result<Option<JSON<Vec<PlanLeg>>>, SpinnerError> {

    match send_channel_plan_query_msg(id, &comm) {
        Ok(plan) => Ok(Some(JSON(plan))),
        Err(_) => Err(SpinnerError),
    }
}

#[put("/channels/<id>/plan", format = "application/json", data = "<new_plan>")]
fn set_plan<'a>(id: u64,
                new_plan: JSON<Vec<PlanLeg>>,
                comm: comm::CommChannelTx)
                -> Result<Response<'a>, SpinnerError> {

    let response_str = send_msg(OutgoingMessage::SetPlan(id, new_plan.into_inner()).into(),
                                &comm)?;

    match response_str.parse() {
        Ok(IncomingMessage::Ret(ret_val)) => {
            if ret_val == 0 {
                Ok(Response::build().status(Status::Accepted).finalize())
            } else {
                Ok(Response::build().status(Status::Conflict).finalize())
            }
        }
        _ => Err(SpinnerError),
    }
}



pub fn get_routes() -> Vec<Route> {

                 routes![query_channels,
                         query_channel,
                         query_channel_state,
                         set_channel_state,
                         query_plan,
                         set_plan]
}

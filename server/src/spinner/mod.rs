use rocket::Route;

use rocket::http::Status;
use rocket::Response;

use rocket::State;

use rocket_contrib::json::Json;

use std::str::FromStr;

use std::collections::HashMap;

use std::time::Duration;

use crate::comm;

#[derive(Debug)]
struct SpinnerError;

type SpinnerResponse<T> = Result<Json<T>, SpinnerError>;

pub enum OutgoingMessage {
    SetState(u8, ChannelCommand),
    GetState(u8),
    SetPlan(u8, Vec<PlanLeg>),
    GetPlan(u8),
}

impl From<OutgoingMessage> for String {
    fn from(msg: OutgoingMessage) -> String {
        match msg {
            OutgoingMessage::SetState(channel_id, ref cmd) => format!(
                "SPINNER,SET_STATE,{},{}",
                channel_id,
                match *cmd {
                    ChannelCommand::Start => "RUNNING",
                    ChannelCommand::Stop => "STOPPED",
                }
            ),
            OutgoingMessage::GetState(channel_id) => format!("SPINNER,GET_STATE,{}", channel_id),
            OutgoingMessage::SetPlan(channel_id, ref plan) => {
                let mut msg = format!("SPINNER,SET_PLAN,{}", channel_id);
                for leg in plan {
                    msg = format!("{},{},{}", msg, leg.duration_msecs, leg.target_val_pct);
                }

                msg
            }
            OutgoingMessage::GetPlan(channel_id) => format!("SPINNER,GET_PLAN,{}", channel_id),
        }
    }
}

enum IncomingMessage {
    Plan(u8, Vec<PlanLeg>),
    State(u8, ChannelState),
    RetVal(i32),
}

impl FromStr for IncomingMessage {
    type Err = SpinnerError;

    fn from_str(s: &str) -> Result<IncomingMessage, Self::Err> {
        let mut tokens = s.split(',');

        if tokens.next() != Some("SPINNER") {
            return Err(SpinnerError);
        }

        if let Some(msg_type) = tokens.next() {
            match msg_type {
                "PLAN_REPLY" => {
                    let ch_num = tokens
                        .next()
                        .ok_or(SpinnerError)?
                        .parse()
                        .map_err(|_| SpinnerError)?;

                    let mut plan_legs = Vec::new();

                    let mut plan_tokens = tokens.peekable();

                    while plan_tokens.peek().is_some() {
                        let duration_msecs = plan_tokens
                            .next()
                            .ok_or(SpinnerError)?
                            .parse()
                            .map_err(|_| SpinnerError)?;
                        let target_val_pct = plan_tokens
                            .next()
                            .ok_or(SpinnerError)?
                            .parse()
                            .map_err(|_| SpinnerError)?;

                        plan_legs.push(PlanLeg {
                            target_val_pct,
                            duration_msecs,
                        });
                    }

                    Ok(IncomingMessage::Plan(ch_num, plan_legs))
                }
                "STATE_REPLY" => {
                    let ch_num = tokens
                        .next()
                        .ok_or(SpinnerError)?
                        .parse()
                        .map_err(|_| SpinnerError)?;

                    let state_str = tokens.next().ok_or(SpinnerError)?;

                    let pos_in_plan_msecs = tokens
                        .next()
                        .ok_or(SpinnerError)?
                        .parse()
                        .map_err(|_| SpinnerError)?;
                    let output_val_pct = tokens
                        .next()
                        .ok_or(SpinnerError)?
                        .parse()
                        .map_err(|_| SpinnerError)?;

                    match state_str {
                        "RUNNING" => Ok(IncomingMessage::State(
                            ch_num,
                            ChannelState::Running {
                                output_val_pct,
                                pos_in_plan_msecs,
                            },
                        )),
                        "STOPPED" => Ok(IncomingMessage::State(ch_num, ChannelState::Stopped)),
                        _ => Err(SpinnerError),
                    }
                }
                "RET_VAL" => {
                    let ret_val = tokens
                        .next()
                        .ok_or(SpinnerError)?
                        .parse()
                        .map_err(|_| SpinnerError)?;
                    Ok(IncomingMessage::RetVal(ret_val))
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

const NUM_CHANNELS: u8 = 1;

fn send_msg(msg_str: String, comm: &comm::CommChannelTx) -> Result<String, SpinnerError> {
    debug!("Sending: {}", msg_str);

    let response_channel = comm.send(msg_str).map_err(|e| {
        warn!("{}", e);
        SpinnerError
    })?;

    let raw_response_msg = response_channel
        .recv_timeout(Duration::from_secs(3))
        .map_err(|e| {
            warn!("{}", e);
            SpinnerError
        })?;

    debug!("Response: {}", raw_response_msg);

    Ok(raw_response_msg)
}

fn send_channel_state_query_msg(
    id: u8,
    comm: &comm::CommChannelTx,
) -> Result<ChannelState, SpinnerError> {
    let response_str = send_msg(OutgoingMessage::GetState(id).into(), comm)?;

    match response_str.parse()? {
        IncomingMessage::State(ret_id, ref state) if ret_id == id => Ok(state.clone()),
        _ => Err(SpinnerError),
    }
}

fn send_channel_plan_query_msg(
    id: u8,
    comm: &comm::CommChannelTx,
) -> Result<Vec<PlanLeg>, SpinnerError> {
    let response_str = send_msg(OutgoingMessage::GetPlan(id).into(), comm)?;

    match response_str.parse()? {
        IncomingMessage::Plan(ret_id, ref plan) if ret_id == id => Ok(plan.clone()),
        _ => Err(SpinnerError),
    }
}

#[get("/channels")]
fn query_channels(
    comm_state: State<comm::CommState>,
) -> SpinnerResponse<HashMap<u8, (ChannelState, Vec<PlanLeg>)>> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    let mut map = HashMap::new();

    for ch_num in 0..NUM_CHANNELS {
        map.insert(
            ch_num,
            (
                send_channel_state_query_msg(ch_num, &comm)?,
                send_channel_plan_query_msg(ch_num, &comm)?,
            ),
        );
    }

    Ok(Json(map))
}

#[get("/channels/<id>")]
fn query_channel(
    id: u8,
    comm_state: State<comm::CommState>,
) -> SpinnerResponse<(ChannelState, Vec<PlanLeg>)> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    Ok(Json((
        send_channel_state_query_msg(id, &comm)?,
        send_channel_plan_query_msg(id, &comm)?,
    )))
}

#[get("/channels/<id>/state")]
fn query_channel_state(
    id: u8,
    comm_state: State<comm::CommState>,
) -> SpinnerResponse<ChannelState> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    match send_channel_state_query_msg(id, &comm) {
        Ok(state) => Ok(Json(state)),
        Err(_) => Err(SpinnerError),
    }
}

#[post(
    "/channels/<id>/state",
    format = "application/json",
    data = "<new_state>"
)]
fn set_channel_state<'a>(
    id: u8,
    new_state: Json<ChannelCommand>,
    comm_state: State<comm::CommState>,
) -> Result<Response<'a>, SpinnerError> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    let response_str = send_msg(
        OutgoingMessage::SetState(id, new_state.into_inner()).into(),
        &comm,
    )?;

    match response_str.parse() {
        Ok(IncomingMessage::RetVal(ret_val)) => {
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
fn query_plan(id: u8, comm_state: State<comm::CommState>) -> SpinnerResponse<Vec<PlanLeg>> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    match send_channel_plan_query_msg(id, &comm) {
        Ok(plan) => Ok(Json(plan)),
        Err(_) => Err(SpinnerError),
    }
}

#[put(
    "/channels/<id>/plan",
    format = "application/json",
    data = "<new_plan>"
)]
fn set_plan<'a>(
    id: u8,
    new_plan: Json<Vec<PlanLeg>>,
    comm_state: State<comm::CommState>,
) -> Result<Response<'a>, SpinnerError> {
    let comm = comm_state.get_comm_channel(0).map_err(|_| SpinnerError)?;

    let response_str = send_msg(
        OutgoingMessage::SetPlan(id, new_plan.into_inner()).into(),
        &comm,
    )?;

    match response_str.parse() {
        Ok(IncomingMessage::RetVal(ret_val)) => {
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
    routes![
        query_channels,
        query_channel,
        query_channel_state,
        set_channel_state,
        query_plan,
        set_plan
    ]
}

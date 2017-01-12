use rocket::Route;

use rocket::Response;
use rocket::http::Status;

use rocket_contrib::JSON;

use std::collections::HashMap;


#[derive(Serialize)]
enum ChannelState {
    Running {
        output_val_pct: f64,
        pos_in_plan_msecs: u64,
    },
    Stopped,
}

#[derive(Deserialize)]
enum ChannelCommand {
    Start,
    Stop,
}

#[derive(Serialize, Deserialize)]
struct PlanLeg {
    target_val_pct: f64,
    duration_msecs: u64,
}


#[get("/channels")]
fn query_channels() -> JSON<HashMap<u64, (ChannelState, Vec<PlanLeg>)>> {
    JSON(HashMap::new())
}

#[get("/channels/<id>")]
fn query_channel(id: u64) -> Option<JSON<ChannelState>> {
    None
}

#[get("/channels/<id>/state")]
fn query_channel_state(id: u64) -> Option<JSON<ChannelState>> {
    None
}

#[post("/channels/<id>/state", format = "application/json", data = "<new_state>")]
fn set_channel_state<'a>(id: u64, new_state: JSON<ChannelCommand>) -> Response<'a> {
    Response::build().status(Status::NotModified).finalize()
}

#[get("/channels/<id>/plan")]
fn query_plan(id: u64) -> Option<JSON<Vec<PlanLeg>>> {
    None
}

#[put("/channels/<id>/plan", format = "application/json", data = "<new_plan>")]
fn set_plan<'a>(id: u64, new_plan: JSON<Vec<PlanLeg>>) -> Response<'a> {
    Response::build().status(Status::NotModified).finalize()
}



pub fn get_routes() -> Vec<Route> {
    routes![query_channels,
            query_channel,
            query_channel_state,
            set_channel_state,
            query_plan,
            set_plan]
}

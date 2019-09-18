use std::collections::HashMap;

use std::borrow::Borrow;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use dotenv;

use clap::{arg_enum, crate_version, value_t_or_exit, App, AppSettings, Arg};

use prettytable as pt;
use pt::{cell, row};

use ratfist_server::db::models::Node;
use ratfist_server::meteo::models::{Sensor, SensorTypeEnum};

/// Prints the supplied data to STDOUT as a formatted ASCII table
fn print_table(mut title_row: pt::Row, table_rows: Vec<pt::Row>) {
    let was_empty = table_rows.is_empty();
    let table_width = title_row.len();

    let mut table = pt::Table::init(table_rows);

    for title_cell in title_row.iter_mut() {
        title_cell.style(pt::Attr::ForegroundColor(pt::color::BLUE));
    }

    table.set_titles(title_row);

    if was_empty {
        let row = pt::Row::new(vec![pt::Cell::new_align(
            "(none)",
            pt::format::Alignment::CENTER,
        )
        .with_hspan(table_width)]);
        table.add_row(row);
    }

    table.printstd();
}

// Returns a vector of all sensor nodes in the database
fn get_node_list(db_conn: &SqliteConnection) -> Vec<Node> {
    use ratfist_server::db::schema::nodes::dsl::*;

    nodes.load::<Node>(db_conn).unwrap()
}

/// Returns an Ok Result with a list of all sensors for a given node, or an Err
/// if no such node exists.
fn get_sensor_list(db_conn: &SqliteConnection, public_node_id: i32) -> Result<Vec<Sensor>, ()> {
    let nid = {
        use ratfist_server::db::schema::nodes::dsl::*;

        nodes
            .filter(public_id.eq(public_node_id))
            .first::<Node>(db_conn)
            .map_err(|_| ())?
            .id
    };

    let sensors = {
        use ratfist_server::meteo::schema::sensors::dsl::*;
        sensors
            .filter(node_id.eq(nid))
            .load::<Sensor>(db_conn)
            .map_err(|_| ())?
    };

    Ok(sensors)
}

// Prints a table with all sensor nodes
fn list_all_nodes(db_conn: &SqliteConnection) {
    let nodes = get_node_list(db_conn);

    print_table(
        row!["Public ID", "Name", "Route Type", "Route Type Parameters"],
        nodes
            .into_iter()
            .map(|node| {
                row![
                    node.public_id,
                    node.name,
                    node.route_type,
                    node.route_param.unwrap_or("".to_string())
                ]
            })
            .collect(),
    );
}

fn list_sensors_in_node(db_conn: &SqliteConnection, node_id: i32) {
    match get_sensor_list(db_conn, node_id) {
        Ok(sensors) => {
            println!("Sensors in node {}:", node_id);
            print_table(
                row!["Public ID", "Type", "Name"],
                sensors
                    .into_iter()
                    .map(|sensor| {
                        row![
                            sensor.public_id,
                            <SensorTypeEnum as Borrow<str>>::borrow(&sensor.sensor_type),
                            sensor.name
                        ]
                    })
                    .collect(),
            )
        }
        Err(_) => println!("No node with ID {} found.", node_id),
    }
}

arg_enum! {
    #[derive(Debug)]
    enum RouteTypes {
        Serial
    }
}

arg_enum! {
    #[derive(Debug)]
    enum SensorTypes {
        Pressure,
        Temperature,
        Humidity,
        LightLevel
    }
}

fn is_positive_integer_i32(arg: String) -> Result<(), String> {
    let err_string = format!("must be a positive integer in [0, {}]", std::i32::MAX);

    let val = arg.parse::<i32>().map_err(|_| err_string.clone())?;

    if val < 0 {
        Err(err_string)
    } else {
        Ok(())
    }
}

fn main() {
    let matches = App::new("meteo_cli")
        .version(crate_version!())
        .subcommands(vec![
            App::new("list")
                .subcommands(vec![
                    App::new("nodes"),
                    App::new("sensors").arg(
                        Arg::with_name("node_public_id")
                            .required(true)
                            .validator(is_positive_integer_i32),
                    ),
                ])
                .setting(AppSettings::SubcommandRequiredElseHelp),
            App::new("add")
                .subcommands(vec![
                    App::new("node").args(&vec![
                        Arg::with_name("public_id")
                            .required(true)
                            .validator(is_positive_integer_i32),
                        Arg::with_name("name").required(true),
                        Arg::with_name("route_type")
                            .required(true)
                            .possible_values(&RouteTypes::variants()),
                    ]),
                    App::new("sensor").args(&vec![
                        Arg::with_name("node_public_id")
                            .required(true)
                            .validator(is_positive_integer_i32),
                        Arg::with_name("sensor_public_id")
                            .required(true)
                            .validator(is_positive_integer_i32),
                        Arg::with_name("sensor_type")
                            .required(true)
                            .possible_values(&SensorTypes::variants()),
                        Arg::with_name("name").required(true),
                    ]),
                ])
                .setting(AppSettings::SubcommandRequiredElseHelp),
        ])
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let db_url = dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env variable");

    let db_conn = SqliteConnection::establish(&db_url)
        .expect(&format!("failed to connect to DB: {}", db_url));

    match matches.subcommand() {
        ("list", Some(list_matches)) => match list_matches.subcommand() {
            ("nodes", _) => list_all_nodes(&db_conn),
            ("sensors", Some(sensors_matches)) => {
                let node_id = value_t_or_exit!(sensors_matches, "node_public_id", i32);

                list_sensors_in_node(&db_conn, node_id);
            }
            _ => unreachable!(),
        },
        ("add", Some(add_matches)) => match add_matches.subcommand() {
            ("node", Some(_node_matches)) => {
                unimplemented!("Adding new nodes not implemented yet.")
            }
            ("sensor", Some(_sensor_matches)) => {
                unimplemented!("Adding new sensors not implemented yet.")
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;

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

/// Returns a Result with a vector of all sensor nodes in the database
fn db_get_node_list(db_conn: &SqliteConnection) -> Result<Vec<Node>, DieselError> {
    use ratfist_server::db::schema::nodes::dsl::*;

    nodes.load::<Node>(db_conn)
}

/// Prints a table with all sensor nodes
fn list_all_nodes(db_conn: &SqliteConnection) {
    let nodes = db_get_node_list(db_conn).expect("database access error");

    print_table(
        row!["Public ID", "Name", "Route Type", "Route Type Parameters"],
        nodes
            .into_iter()
            .map(|node| {
                row![
                    node.public_id,
                    node.name,
                    node.route_type,
                    node.route_param.unwrap_or_else(|| "".to_string())
                ]
            })
            .collect(),
    );
}

/// Returns an Ok Result with a list of all sensors for a given node, or an Err
/// if no such node exists.
fn db_get_sensor_list(
    db_conn: &SqliteConnection,
    public_node_id: i32,
) -> Result<Vec<Sensor>, DieselError> {
    let nid = {
        use ratfist_server::db::schema::nodes::dsl::*;

        nodes
            .filter(public_id.eq(public_node_id))
            .first::<Node>(db_conn)?
            .id
    };

    let sensors = {
        use ratfist_server::meteo::schema::sensors::dsl::*;
        sensors.filter(node_id.eq(nid)).load::<Sensor>(db_conn)?
    };

    Ok(sensors)
}

/// Prints a table with the sensors for a given node ID
fn list_sensors_in_node(db_conn: &SqliteConnection, node_id: i32) {
    match db_get_sensor_list(db_conn, node_id) {
        Ok(sensors) => {
            println!("Sensors in node {}:", node_id);
            print_table(
                row!["Public ID", "Type", "Name"],
                sensors
                    .into_iter()
                    .map(|sensor| row![sensor.public_id, sensor.sensor_type.as_ref(), sensor.name])
                    .collect(),
            )
        }
        Err(DieselError::NotFound) => println!("No node with ID {} found.", node_id),
        Err(other_err) => {
            panic!("Unhandled error: {:?}", other_err);
        }
    }
}

/// Adds a new sensor node with the given paramenters.
fn db_add_node(
    db_conn: &SqliteConnection,
    node_id: i32,
    node_name: &str,
    route_type: RouteTypes,
    route_param_str: &str,
) -> Result<(), DieselError> {
    let route_type_str: &str = route_type.as_ref();

    {
        use ratfist_server::db::schema::nodes::dsl::*;

        insert_into(nodes)
            .values((
                public_id.eq(node_id),
                name.eq(node_name),
                route_type.eq(route_type_str),
                route_param.eq(route_param_str),
            ))
            .execute(db_conn)
            .map(|_| ())
    }
}

/// Adds a new sensor node and handles the result of the DB operation.
fn add_node(
    db_conn: &SqliteConnection,
    node_id: i32,
    node_name: &str,
    route_type: RouteTypes,
    route_params: Option<&str>,
) {
    match db_add_node(
        db_conn,
        node_id,
        node_name,
        route_type,
        route_params.unwrap_or(""),
    ) {
        Ok(_) => {
            println!(
                "Succesfully created new sensor node: public_id {}, name '{}', route type {}, route params {:?}",
                node_id,
                node_name,
                route_type.as_ref(),
                route_params
            );
        }
        Err(DieselError::DatabaseError(error_kind, error_details)) => {
            println!(
                "Failed to add new sensor node because of a DB error: {:?}",
                error_kind
            );
            println!("Error details: {:?}", error_details);
        }
        Err(other_err) => {
            panic!("Unhandled error: {:?}", other_err);
        }
    }
}

/// Adds a new sensor to a given sensor node.
fn db_add_sensor(
    db_conn: &SqliteConnection,
    parent_node_id: i32,
    sensor_id: i32,
    sensor_name: &str,
    sensor_type: SensorTypes,
) -> Result<(), DieselError> {
    let sensor_type_enum: SensorTypeEnum = sensor_type.into();

    let nid = {
        use ratfist_server::db::schema::nodes::dsl::*;

        nodes
            .filter(public_id.eq(parent_node_id))
            .first::<Node>(db_conn)?
            .id
    };

    {
        use ratfist_server::meteo::schema::sensors::dsl::*;

        insert_into(sensors)
            .values((
                public_id.eq(sensor_id),
                node_id.eq(nid),
                name.eq(sensor_name),
                sensor_type.eq(sensor_type_enum),
                name.eq(sensor_name),
            ))
            .execute(db_conn)
            .map(|_| ())
    }
}

/// Adds a new sensor to a given sensor node, and handles the result of the DB
/// operation.
fn add_sensor(
    db_conn: &SqliteConnection,
    parent_node_id: i32,
    sensor_id: i32,
    sensor_name: &str,
    sensor_type: SensorTypes,
) {
    match db_add_sensor(db_conn, parent_node_id, sensor_id, sensor_name, sensor_type) {
        Ok(_) => {
            println!(
                "Succesfully created new sensor for node #{}: public_id {}, name '{}', type {}",
                parent_node_id,
                sensor_id,
                sensor_name,
                SensorTypeEnum::from(sensor_type).as_ref()
            );
        }
        Err(DieselError::DatabaseError(error_kind, error_details)) => {
            println!(
                "Failed to add new sensor because of a DB error: {:?}",
                error_kind
            );
            println!("Error details: {:?}", error_details);
        }
        Err(other_err) => {
            panic!(other_err);
        }
    }
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    enum RouteTypes {
        Serial,
        EnviroPHat
    }
}

impl AsRef<str> for RouteTypes {
    fn as_ref(&self) -> &'static str {
        match &self {
            RouteTypes::Serial => "serial",
            RouteTypes::EnviroPHat => "envirophat",
        }
    }
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    enum SensorTypes {
        Pressure,
        Temperature,
        Humidity,
        LightLevel
    }
}

impl From<SensorTypes> for SensorTypeEnum {
    fn from(sensor_type: SensorTypes) -> SensorTypeEnum {
        match sensor_type {
            SensorTypes::Pressure => SensorTypeEnum::Pressure,
            SensorTypes::Temperature => SensorTypeEnum::Temperature,
            SensorTypes::Humidity => SensorTypeEnum::Humidity,
            SensorTypes::LightLevel => SensorTypeEnum::Humidity,
        }
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
                    App::new("node").args(&[
                        Arg::with_name("public_id")
                            .required(true)
                            .validator(is_positive_integer_i32),
                        Arg::with_name("name").required(true),
                        Arg::with_name("route_type")
                            .required(true)
                            .possible_values(&RouteTypes::variants()),
                        Arg::with_name("route_params"),
                    ]),
                    App::new("sensor").args(&[
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
        .unwrap_or_else(|_| panic!("failed to connect to DB: {}", db_url));

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
            ("node", Some(node_matches)) => {
                let node_id = value_t_or_exit!(node_matches, "public_id", i32);
                let node_name = node_matches
                    .value_of("name")
                    .expect("missing new node name");
                let route_type = value_t_or_exit!(node_matches, "route_type", RouteTypes);

                let route_params = match route_type {
                    RouteTypes::Serial | RouteTypes::EnviroPHat => {
                        let param_str =
                            node_matches.value_of("route_params").unwrap_or_else(|| {
                                panic!("route_params parameter is required with route_type Serial")
                            });

                        is_positive_integer_i32(param_str.to_string())
                            .unwrap_or_else(|s| panic!("route_params validation error: {}", s));

                        Some(param_str)
                    }
                };

                add_node(&db_conn, node_id, node_name, route_type, route_params);
            }
            ("sensor", Some(sensor_matches)) => {
                let node_id = value_t_or_exit!(sensor_matches, "node_public_id", i32);
                let sensor_id = value_t_or_exit!(sensor_matches, "sensor_public_id", i32);
                let sensor_name = sensor_matches
                    .value_of("name")
                    .expect("missing new sensor name");
                let sensor_type = value_t_or_exit!(sensor_matches, "sensor_type", SensorTypes);

                add_sensor(&db_conn, node_id, sensor_id, sensor_name, sensor_type);
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

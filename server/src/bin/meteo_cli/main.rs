use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use dotenv;

use clap::{clap_app, crate_version, AppSettings};

use prettytable as pt;
use pt::{cell, row};

use ratfist_server::db::models::Node;

fn get_node_list(db_conn: &SqliteConnection) -> Vec<Node> {
    use ratfist_server::db::schema::nodes::dsl::*;

    nodes.load::<Node>(db_conn).unwrap()
}

fn list_all_nodes(db_conn: &SqliteConnection) {
    let nodes = get_node_list(db_conn);

    let mut table = pt::Table::new();

    table.set_titles(row![
        Fbb =>
        "Public ID",
        "Name",
        "Route Type",
        "Route Type Parameters"
    ]);

    if nodes.is_empty() {
        table.add_row(row![
            H4c =>
            "(none)"
        ]);
    }

    for node in nodes {
        table.add_row(row![
            node.public_id,
            node.name,
            node.route_type,
            node.route_param.unwrap_or("".to_string())
        ]);
    }

    table.printstd();
}

fn main() {
    let matches = clap_app!(meteo_cli =>
        (version: crate_version!())
        (@subcommand list =>
        )
    )
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .get_matches();

    let db_url = dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env variable");

    let db_conn = SqliteConnection::establish(&db_url)
        .expect(&format!("failed to connect to DB: {}", db_url));

    match matches.subcommand() {
        ("list", _) => list_all_nodes(&db_conn),
        _ => panic!(),
    }
}

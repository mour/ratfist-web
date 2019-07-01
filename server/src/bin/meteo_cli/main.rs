
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use dotenv;

use ratfist_server::db::models::Node;


fn main() {
    let db_url = dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env variable");

    let db_conn = SqliteConnection::establish(&db_url)
                        .expect(&format!("failed to connect to DB: {}", db_url));

    let nodes = {
        use ratfist_server::db::schema::nodes::dsl::*;

        nodes
            .load::<Node>(&db_conn)
            .unwrap()
    };

    println!("Nodes: {:?}", nodes);
}

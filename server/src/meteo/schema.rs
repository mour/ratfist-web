use crate::db::schema::nodes;

table! {
    measurements (id) {
        id -> Integer,
        sensor_id -> Integer,
        value -> Float,
        measured_at -> BigInt,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        public_id -> Integer,
        node_id -> Integer,
        type_id -> Integer,
        name -> Text,
    }
}

joinable!(measurements -> sensors (sensor_id));
joinable!(sensors -> nodes (node_id));

allow_tables_to_appear_in_same_query!(measurements, nodes, sensors,);

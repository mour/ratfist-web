table! {
    measurements (id) {
        id -> Integer,
        sensor_id -> Integer,
        value -> Float,
        measured_at -> Integer,
    }
}

table! {
    nodes (id) {
        id -> Integer,
        public_id -> Integer,
        name -> Text,
        route_type -> Text,
        route_param -> Nullable<Text>,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        public_id -> Integer,
        node_id -> Integer,
        sensor_type -> Integer,
        name -> Text,
    }
}

joinable!(measurements -> sensors (sensor_id));
joinable!(sensors -> nodes (node_id));

allow_tables_to_appear_in_same_query!(
    measurements,
    nodes,
    sensors,
);

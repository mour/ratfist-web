table! {
    measurements (id) {
        id -> Integer,
        sensor_id -> Integer,
        value -> Float,
        measured_at -> BigInt,
    }
}

table! {
    sensor_types (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        public_id -> Integer,
        type_id -> Integer,
        name -> Text,
    }
}

joinable!(measurements -> sensors (sensor_id));
joinable!(sensors -> sensor_types (type_id));

allow_tables_to_appear_in_same_query!(measurements, sensor_types, sensors,);

use norgopolis_module::module_communication::MessagePack;
use surrealdb::sql::Value;

pub fn value_to_msgpack(value: &Value) -> MessagePack {
    let mut bytes = Vec::new();

    let transcoded = transcode_value_msgpack(value);
    rmpv::encode::write_value(&mut bytes, &transcoded).unwrap();

    MessagePack { data: bytes }
}

fn transcode_value_msgpack(value: &Value) -> rmpv::Value {
    match value {
        Value::None | Value::Null => rmpv::Value::Nil,
        Value::Bool(bool) => rmpv::Value::Boolean(*bool),
        Value::Number(number) => rmpv::Value::Integer(number.to_int().into()),
        Value::Strand(strand) => rmpv::Value::String(strand.clone().to_raw().into()),
        Value::Duration(duration) => rmpv::Value::Map(Vec::from([(
            "duration".into(),
            duration.as_secs_f64().into(),
        )])),
        Value::Datetime(datetime) => rmpv::Value::Map(Vec::from([(
            "datetime".into(),
            datetime.timestamp_micros().into(),
        )])),
        Value::Uuid(uuid) => uuid.as_bytes().as_slice().into(),
        Value::Array(array) => array
            .iter()
            .map(transcode_value_msgpack)
            .collect(),
        Value::Object(object) => rmpv::Value::Map(
            object
                .iter()
                .map(|(k, v)| (k.clone().into(), transcode_value_msgpack(v)))
                .collect(),
        ),
        Value::Geometry(geometry) => transcode_value_msgpack(&geometry.as_coordinates()),
        Value::Bytes(bytes) => bytes.to_string().into(),
        Value::Thing(thing) => rmpv::Value::Map(Vec::from([
            ("id".into(), thing.id.to_string().into()),
            ("tb".into(), thing.tb.clone().into()),
        ])),
        // As per the documentation other types will be coerced back into simpler types
        // before being sent back to the client and so we shouldn't worry about them here.
        _ => unimplemented!(),
    }
}

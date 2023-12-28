use anyhow::Result;
use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Code, Module, Status,
};
use std::path::Path;
use tokio_stream::wrappers::ReceiverStream;
use value_to_msgpack_transcoder::value_to_msgpack;

use surrealdb::engine::any::{connect, Any as AnySurrealConnection};
use surrealdb::Surreal;

mod value_to_msgpack_transcoder;

struct Norgberg {
    connection: Surreal<AnySurrealConnection>,
}

impl Norgberg {
    async fn new(file: &Path) -> Result<Self> {
        let connection = connect(file.to_str().unwrap_or("memory")).await?;

        connection
            .use_ns("neorg")
            .use_db("neorg")
            .await
            .expect("Failed to connect to db");

        connection
            .query("DEFINE NAMESPACE neorg; DEFINE DATABASE NEORG;")
            .await
            .expect("Unable to create neorg namespace nor db!");

        Ok(Norgberg { connection })
    }
}

#[norgopolis_module::async_trait]
impl Service for Norgberg {
    type Stream = ReceiverStream<Result<MessagePack, Status>>;

    async fn call(
        &self,
        fn_name: String,
        args: Option<MessagePack>,
    ) -> Result<Self::Stream, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        match fn_name.as_str() {
            "execute-query" => match args {
                Some(arg) => {
                    let query = match arg.decode::<String>().map_err(|err| {
                        Status::new(
                            Code::InvalidArgument,
                            "Invalid argument provided! Expected type `string`: ".to_string()
                                + &err.to_string(),
                        )
                    }) {
                        Ok(val) => val,
                        Err(err) => {
                            tx.send(Err(err.clone())).await.unwrap();
                            return Err(err);
                        }
                    };

                    match self
                        .connection
                        .query(query)
                        .await
                        .map_err(|err| Status::new(Code::Cancelled, err.to_string()))
                    {
                        Ok(mut val) => {
                            for i in 0..val.num_statements() {
                                match val.take(i) {
                                    Ok(val) => {
                                        tx.send(Ok(value_to_msgpack(&val))).await.unwrap();
                                    }
                                    Err(err) => {
                                        tx.send(Err(Status::new(Code::Cancelled, err.to_string())))
                                            .await
                                            .unwrap();
                                        return Ok(ReceiverStream::new(rx));
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            tx.send(Err(err.clone())).await.unwrap();
                            return Err(err);
                        }
                    };
                }
                None => {
                    tx.send(Err(Status::new(
                        Code::InvalidArgument,
                        "Expected a string as argument, whereas nothing was provided instead.",
                    )))
                    .await
                    .unwrap();
                }
            },
            _ => todo!(),
            // "execute-live-query" => {},
        };

        Ok(ReceiverStream::new(rx))
    }
}

#[tokio::main]
async fn main() {
    let database_location = if cfg!(debug_assertions) {
        "memory".to_owned()
    } else {
        let data_dir = directories::ProjectDirs::from("org", "neorg", "norgberg").expect("Could not grab known data directories, are you running on a non-unix and non-windows system?").data_dir().to_path_buf();

        let _ = std::fs::create_dir_all(&data_dir);

        "file://".to_string() + data_dir.join("database.sql").to_str().unwrap()
    };

    Module::start(
        Norgberg::new(Path::new(&database_location))
            .await
            .expect("Unable to connect to database!"),
    )
    .await
    .unwrap()
}

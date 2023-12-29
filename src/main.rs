use anyhow::Result;
use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Code, Module, Status,
};
use std::path::Path;
use surrealdb::engine::local::{Db, File};
use tokio_stream::wrappers::ReceiverStream;
use value_to_msgpack_transcoder::value_to_msgpack;

use surrealdb::Surreal;

mod value_to_msgpack_transcoder;

struct Norgberg {
    connection: Surreal<Db>,
}

impl Norgberg {
    async fn new(file: &Path) -> Result<Self> {
        let connection = Surreal::new::<File>(file.to_str().unwrap_or("memory")).await?;

        connection
            .use_ns("neorg")
            .use_db("neorg")
            .await
            .expect("Failed to connect to db");

        connection
            .query("DEFINE NAMESPACE neorg; DEFINE DATABASE NEORG;")
            .await
            .expect("Unable to create neorg namespace nor db!");

        connection
            .query(
                "
            BEGIN TRANSACTION;
            IF schema:current != NULL {
                RETURN;
            };

            CREATE schema:current SET version = '0.0.1';

            IF !string::is::semver(schema:current.version) {
                THROW 'Attempted to create schema with invalid semver version!';
            };

            DEFINE TABLE files SCHEMAFULL;
            DEFINE INDEX path ON TABLE files COLUMNS path UNIQUE;
            DEFINE FIELD path ON TABLE files TYPE string;
            // TODO: DEFINE FIELD metadata

            DEFINE TABLE vFiles SCHEMAFULL;
            DEFINE INDEX path ON TABLE vFiles COLUMNS path UNIQUE; 
            DEFINE FIELD path ON TABLE vFiles TYPE string; 

            DEFINE TABLE externalResources SCHEMAFULL;
            DEFINE INDEX id ON TABLE externalResources COLUMNS id UNIQUE;
            DEFINE FIELD id ON TABLE externalResources TYPE string;
            DEFINE FIELD type ON TABLE externalResources TYPE string
              ASSERT $value INSIDE ['uri', 'file'];
            COMMIT TRANSACTION;
        ",
            )
            .await
            .expect("Unable to configure database!");

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

        data_dir.join("norgberg.db").to_str().unwrap().to_string()
    };

    Module::start(
        Norgberg::new(Path::new(&database_location))
            .await
            .expect("Unable to connect to database!"),
    )
    .await
    .unwrap()
}

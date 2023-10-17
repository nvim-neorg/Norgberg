use anyhow::Result;
use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Module, Status,
};
use std::path::Path;
use tokio_stream::wrappers::ReceiverStream;

use surrealdb::engine::any::{connect, Any as AnySurrealConnection};
use surrealdb::Surreal;

struct Norgberg {
    connection: Surreal<AnySurrealConnection>,
}

impl Norgberg {
    async fn new(file: &Path) -> Result<Self> {
        Ok(Norgberg {
            connection: connect(file.to_str().unwrap_or("memory")).await?,
        })
    }
}

#[norgopolis_module::async_trait]
impl Service for Norgberg {
    type Stream = ReceiverStream<Result<MessagePack, Status>>;

    async fn call(
        &self,
        _fn_name: String,
        _args: Option<MessagePack>,
    ) -> Result<Self::Stream, Status> {
        todo!()
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
        Norgberg::new(&Path::new(&database_location))
            .await
            .expect("Unable to connect to database!"),
    )
    .await
    .unwrap()
}

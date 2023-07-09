use anyhow::{anyhow, Result};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Module, Status,
};
use std::path::Path;
use tokio_stream::wrappers::ReceiverStream;

struct Norgberg {
    connection: Pool<ConnectionManager<SqliteConnection>>,
}

impl Norgberg {
    fn new(file: &Path) -> Result<Self> {
        Ok(Norgberg {
            connection: Pool::builder().build(ConnectionManager::new(
                file.to_str()
                    .ok_or_else(|| anyhow!("Invalid unicode in path!"))?,
            ))?,
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
    Module::start(Norgberg::new(&Path::new(":memory:")).expect("Unable to connect to database!"))
        .await
        .unwrap()
}

#[cfg(Release)]
#[tokio::main]
async fn main() {
    let data_dir = directories::ProjectDirs::from("org", "neorg", "norgberg").expect("Could not grab known data directories, are you running on a non-unix and non-windows system?").data_dir().to_path_buf();

    let _ = std::fs::create_dir_all(&data_dir);

    Module::start(
        Norgberg::new(&data_dir.join("database.sql")).expect("Unable to connect to database!"),
    )
    .await
    .unwrap()
}

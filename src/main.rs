use anyhow::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Module, Status,
};
use tokio_stream::wrappers::ReceiverStream;

struct Norgberg {
    // TODO: Add connection handle
    connection: Pool<ConnectionManager<SqliteConnection>>,
}

impl Norgberg {
    fn new(file: &str) -> Result<Self> {
        Ok(Norgberg {
            connection: Pool::builder().build(ConnectionManager::new(file))?,
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
    // TODO: Create a path at `~/.local/share/norgberg/database.sql`
    Module::start(Norgberg::new("mydb.sql").expect("Unable to connect to database!"))
        .await
        .unwrap()
}

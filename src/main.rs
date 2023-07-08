use norgopolis_module::{
    invoker_service::Service, module_communication::MessagePack, Module, Status,
};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Default)]
struct Norgberg {
    // TODO: Add connection handle
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
    Module::start(Norgberg::default()).await.unwrap()
}

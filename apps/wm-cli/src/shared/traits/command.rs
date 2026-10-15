use std::future::Future;

pub trait Command {
    fn run(self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
}

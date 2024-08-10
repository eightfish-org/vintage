use async_trait::async_trait;
use tokio::task::JoinHandle;

#[async_trait]
pub trait Service {
    type Input;
    type Output;
    async fn service(self, input: Self::Input) -> Self::Output;
}

pub fn start_service<TService>(
    service: TService,
    input: TService::Input,
) -> JoinHandle<TService::Output>
where
    TService: Service + 'static,
    TService::Output: Send + 'static,
{
    tokio::spawn(service.service(input))
}

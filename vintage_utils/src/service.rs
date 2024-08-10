use async_trait::async_trait;
use tokio::task::JoinHandle;

#[async_trait]
pub trait Service {
    type Output;
    async fn service(self) -> Self::Output;
}

pub fn start_service<TService>(service: TService) -> JoinHandle<TService::Output>
where
    TService: Service + 'static,
    TService::Output: Send + 'static,
{
    tokio::spawn(service.service())
}

use async_trait::async_trait;
use tokio::task::JoinHandle;

#[async_trait]
pub trait Service {
    type Input;
    type Output;
    async fn service(self, input: Self::Input) -> Self::Output;
}

pub struct ServiceStarter<TService>
where
    TService: Service,
{
    service: TService,
    input: TService::Input,
}

impl<TService> ServiceStarter<TService>
where
    TService: Service + 'static,
    TService::Output: Send + 'static,
{
    pub fn new(service: TService) -> Self
    where
        TService::Input: Default,
    {
        Self {
            service,
            input: Default::default(),
        }
    }

    pub fn new_with_input(service: TService, input: TService::Input) -> Self {
        Self { service, input }
    }

    pub fn start(self) -> JoinHandle<TService::Output> {
        tokio::spawn(self.service.service(self.input))
    }
}

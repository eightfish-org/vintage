use crate::build_router;
use async_trait::async_trait;
use hyper::Server;
use std::net::SocketAddr;
use vintage_meta_data::MetaDataDb;
use vintage_utils::{Service, ServiceStarter};

pub struct HttpService {
    addr: SocketAddr,
    state: HttpServiceState,
}

impl HttpService {
    pub fn create(addr: SocketAddr, meta_data_db: MetaDataDb) -> ServiceStarter<Self> {
        ServiceStarter::new(Self {
            addr,
            state: HttpServiceState::new(meta_data_db),
        })
    }
}

#[async_trait]
impl Service for HttpService {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        Server::bind(&self.addr)
            .serve(build_router(self.state).into_make_service())
            .await
            .expect("hyper::Server");
    }
}

#[derive(Clone)]
pub(crate) struct HttpServiceState {
    pub meta_data_db: MetaDataDb,
}

impl HttpServiceState {
    pub fn new(meta_data_db: MetaDataDb) -> Self {
        Self { meta_data_db }
    }
}

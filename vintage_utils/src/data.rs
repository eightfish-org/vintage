use crate::Activation;
use std::sync::Mutex;

pub struct Data<TData> {
    activation: Activation,
    data: Mutex<TData>,
}

impl<TData> Data<TData> {
    pub fn new(data: TData) -> Self {
        Self {
            activation: Activation::new(false),
            data: Mutex::new(data),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.activation.is_active()
    }

    pub fn set_data(&self, data: TData) {
        {
            *self.data.lock().unwrap() = data
        }
        self.activation.set_active(true);
    }

    pub async fn clone_data(&self) -> TData
    where
        TData: Clone,
    {
        self.activation.wait().await;
        {
            self.data.lock().unwrap().clone()
        }
    }

    pub async fn into_data(self) -> TData {
        self.activation.wait().await;
        {
            self.data.into_inner().unwrap()
        }
    }
}

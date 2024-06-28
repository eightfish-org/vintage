pub struct TXPool {
    incoming_queue: Vec<Tx>,
    ready_queue: Vec<Tx>,
    rt: Arc<Mutex<TokioRT>>,
}

impl TXPool {
    pub fn new() -> Self {
        TXPool {
            incoming_queue: Vec::new(),
            ready_queue: Vec::new(),
        }
    }

    pub fn start_service() {
        // use tokio to start a server
        // listening to a channel <- ch_to_txpool
        // other task send raw tx -> ch_to_txpool

        // insert the raw tx to incoming_queue

        // we need to maintain an instance of TXPool in a global Vintage object
    }
}

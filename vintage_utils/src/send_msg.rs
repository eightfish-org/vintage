use tokio::sync::mpsc;

pub trait SendMsg {
    type Msg;
    fn send_msg(&self, msg: Self::Msg) -> bool;
}

impl<TMsg> SendMsg for mpsc::Sender<TMsg> {
    type Msg = TMsg;

    fn send_msg(&self, msg: Self::Msg) -> bool {
        if let Err(err) = self.try_send(msg) {
            log::trace!("mpsc::Sender try_send err: {}", err);
            false
        } else {
            true
        }
    }
}

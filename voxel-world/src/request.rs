use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

pub trait Request {
    type Response;
}

pub struct Call<R: Request> {
    pub payload: R,
    reply_to: oneshot::Sender<R::Response>,
}

impl<R: Request> Call<R> {
    pub fn new(payload: R) -> (Self, oneshot::Receiver<R::Response>) {
        let (tx, rx) = oneshot::channel();
        (
            Call {
                payload,
                reply_to: tx,
            },
            rx,
        )
    }

    pub fn reply(self, response: R::Response) {
        let _ = self.reply_to.send(response);
    }
}

// -------------------- //

#[derive(Clone, Serialize, Deserialize)]
pub struct Connect {
    pub name: String,
}

impl Request for Connect {
    type Response = u32;
}

#[derive(Serialize, Deserialize)]
pub struct Ping;
#[derive(Serialize, Deserialize)]
pub struct Pong;

impl Request for Ping {
    type Response = Pong;
}

pub enum WorldRequest {
    Connect(Call<Connect>),
    Ping(Call<Ping>),
}

use anyhow::{Context, Result};
use rtrb::{Consumer, Producer, RingBuffer};

pub struct BiDirectionalChannel<Tx, Rx> {
    sender: Producer<Tx>,
    receiver: Consumer<Rx>,
}

impl<Tx, Rx> BiDirectionalChannel<Tx, Rx> {
    pub fn send(&mut self, msg: Tx) -> Result<()> {
        self.sender.push(msg).map_err(|_| anyhow::anyhow!("tx queue full!"))
    }

    pub fn recv(&mut self) -> Result<Rx> {
        self.receiver.pop().context("rx queue empty")
    }
}

pub fn create_bi_channel<MsgA, MsgB>(
    capacity: usize
) -> (BiDirectionalChannel<MsgA, MsgB>, BiDirectionalChannel<MsgB, MsgA>) {
    let (prod_a2b, cons_a2b) = RingBuffer::new(capacity);
    let (prod_b2a, cons_b2a) = RingBuffer::new(capacity);

    let side_a = BiDirectionalChannel {
        sender: prod_a2b,
        receiver: cons_b2a,
    };

    let side_b = BiDirectionalChannel {
        sender: prod_b2a,
        receiver: cons_a2b,
    };

    (side_a, side_b)
}


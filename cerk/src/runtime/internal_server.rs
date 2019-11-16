use crate::kernel::BrokerEvent;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};

pub trait InternalServer {
    fn process_broker_event(&mut self, event: BrokerEvent, outbox: &BoxedSender);
    fn start_listening_to_broker(&mut self, inbox: BoxedReceiver, outbox: BoxedSender) {
        loop {
            let broker_event = inbox.receive();
            self.process_broker_event(broker_event, &outbox);
        }
    }
}

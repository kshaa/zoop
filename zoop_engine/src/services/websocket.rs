use ewebsock::{Error, WsEvent, WsMessage, WsReceiver, WsSender};
use ggrs::{Message, NonBlockingSocket};
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};
use zoop_shared::player_id::PlayerId;
use zoop_shared::player_message::PlayerMessage;

/// A simple non-blocking WebSocket connection to use with GGRS Sessions
#[derive(Debug)]
pub struct NonBlockingWebSocket {
    address: String,
    sender: Arc<Mutex<WrappedWsSender>>,
    receiver: Arc<Mutex<WrappedWsReceiver>>,
}

// Might blow up
unsafe impl Send for NonBlockingWebSocket {}
unsafe impl Sync for NonBlockingWebSocket {}

struct WrappedWsSender {
    underlying: WsSender,
    opened: bool,
}
impl WrappedWsSender {
    fn new(underlying: WsSender) -> WrappedWsSender {
        WrappedWsSender {
            underlying,
            opened: false,
        }
    }
}
impl fmt::Debug for WrappedWsSender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedWsSender")
    }
}

struct WrappedWsReceiver {
    underlying: WsReceiver,
}
impl WrappedWsReceiver {
    fn new(underlying: WsReceiver) -> WrappedWsReceiver {
        WrappedWsReceiver { underlying }
    }
}
impl fmt::Debug for WrappedWsReceiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedWsReceiver")
    }
}

impl NonBlockingWebSocket {
    /// Connects to a Websocket address in a non-blocking manner.
    pub fn connect(address: String) -> Result<Self, Error> {
        let options = ewebsock::Options::default();
        let (sender, receiver) = match ewebsock::connect(address.clone(), options) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        let wrapped_sender = Arc::new(Mutex::new(WrappedWsSender::new(sender)));
        let wrapped_receiver = Arc::new(Mutex::new(WrappedWsReceiver::new(receiver)));

        Ok(Self {
            address,
            sender: wrapped_sender,
            receiver: wrapped_receiver,
        })
    }
}

impl NonBlockingSocket<PlayerId> for NonBlockingWebSocket {
    fn send_to(&mut self, msg: &Message, addr: &PlayerId) {
        // GGRS expects an unreliable transport
        // so write failures are just ignored
        if let Ok(mut writer) = self.sender.lock() {
            let message = serde_json::to_string(msg).unwrap();
            let player_message =
                serde_json::to_string(&PlayerMessage::to(addr.clone(), message)).unwrap();
            if writer.opened {
                writer.underlying.send(WsMessage::Text(player_message));
            }
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(PlayerId, Message)> {
        let mut received_messages = Vec::new();

        // This might fail, but no worries, GGRS will try again later
        if let (Ok(mut sender), Ok(receiver)) = (self.sender.lock(), self.receiver.lock()) {
            while let Some(event) = receiver.underlying.try_recv() {
                match event {
                    WsEvent::Message(WsMessage::Binary(e)) => panic!(
                        "Websocket received unexpected binary from {}: {:?}",
                        &self.address, e
                    ),
                    WsEvent::Message(WsMessage::Unknown(e)) => panic!(
                        "Websocket received unknown message from {}: {:?}",
                        &self.address, e
                    ),
                    WsEvent::Message(WsMessage::Pong(p)) => panic!(
                        "Websocket received unexpected pong from {}: {:?}",
                        &self.address, p
                    ),
                    WsEvent::Message(WsMessage::Ping(p)) => {
                        if sender.opened {
                            sender.underlying.send(WsMessage::Pong(p))
                        }
                    }
                    WsEvent::Message(WsMessage::Text(text)) => {
                        let from_player_message: PlayerMessage =
                            serde_json::from_str(text.as_str()).unwrap();
                        let from_address = from_player_message.address;
                        let message: Message =
                            serde_json::from_str(from_player_message.message.as_str()).unwrap();
                        received_messages.push((from_address, message));
                    }
                    WsEvent::Error(e) => panic!("Websocket error for {}: {:?}", &self.address, e),
                    WsEvent::Closed => panic!("Websocket closed for {:?}", &self.address),
                    WsEvent::Opened => sender.opened = true, // Ignore
                }
            }
        }

        received_messages
    }
}

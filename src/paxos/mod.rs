use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::thread::{spawn, JoinHandle};

/// Phase 1. (a) A proposer selects a proposal number n and sends a prepare
/// request with number n to a majority of acceptors.  (b) If an acceptor
/// receives a prepare request with number n greater than that of any prepare
/// request to which it has already responded, then it responds to the request
/// with a promise not to accept any more proposals numbered less than n and
/// with the highest-numbered proposal (if any) that it has accepted.
///
/// Phase 2. (a) If the proposer receives a response to its prepare requests
/// (numbered n) from a majority of acceptors, then it sends an accept request
/// to each of those acceptors for a proposal numbered n with a value v, where
/// v is the value of the highest-numbered proposal among the responses, or is
/// any value if the responses reported no proposals.  (b) If an acceptor
/// receives an accept request for a proposal numbered n, it accepts the
/// proposal unless it has already responded to a prepare request having a
/// number greater than n.

/// Identifier for a worker node.
#[derive(Hash, Serialize, Deserialize, Debug)]
pub struct WorkerId {
    value: usize,
}

/// Identifier for a request.
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RequestNumber {
    value: usize,
}

/// Request for the "Prepare" phase.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrepareRequest {
    sender: WorkerId,
    request_number: RequestNumber,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Proposal {
    request_number: RequestNumber,
    value: usize,
}


/// Request for the "Accept" phase.
#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptRequest {
    sender: WorkerId,
    proposal: Proposal,
}

/// Response to a PrepareRequest.
#[derive(Serialize, Deserialize, Debug)]
pub enum PrepareResponse {
    /// Returned if no proposal has been accepted yet.
    Promise {
        /// The id of the minimum request id we're now willing to accept.
        id: RequestNumber,
        /// The currently-accepted value, if any.
        accepted: Option<Proposal>,
    },

    /// Returned if we've already promised to accept a higher proposal number.
    AlreadyPromised { id: RequestNumber },
}

/// Response to an AcceptRequest
#[derive(Serialize, Deserialize, Debug)]
pub enum AcceptResponse {
    Accepted {
        node: WorkerId,
        request: AcceptRequest,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ExternalRequest(usize),
    Prepare(PrepareRequest),
    Prepared(PrepareResponse),
    Accept(AcceptRequest),
    Accepted(AcceptResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerState {
    /// Our current proposal
    /// The RequestNumber such that we've promised to only accept higher proposal numbers.
    min_acceptable: Option<RequestNumber>,
    /// The highest-numbered request we've accepted.
    accepted: Option<AcceptRequest>,
    current_proposal: Option<Proposal>,
}

impl WorkerState {
    fn empty() -> Self {
        Self {
            min_acceptable: None,
            accepted: None,
            current_proposal: None,
        }
    }
}

/// Message bus for sending messages between workers.
pub struct MessageBus {
    /// Channel end used by other threads to send messages to the bus.
    /// We hand out clones of this to clients.
    address: Sender<(WorkerId, Message)>,
    /// Channel end used to receive messages on the bus.
    inbox: Receiver<(WorkerId, Message)>,
    /// Channels for forwarding messages sent to the bus.
    mailboxes: Vec<Sender<Message>>,
}

impl MessageBus {

    pub fn new() -> Self {
        let (address, inbox) = channel();
        MessageBus {
            address,
            inbox,
            mailboxes: Vec::new(),
        }
    }

    pub fn add_slot(&mut self) -> (WorkerId, Sender<(WorkerId, Message)>, Receiver<Message>) {
        let (sender, receiver) = channel();
        let id = WorkerId {
            value: self.mailboxes.len(),
        };
        self.mailboxes.push(sender);
        (id, self.address.clone(), receiver)
    }

    pub fn run(&self) -> Result<(), SendError<Message>> {
        for (id, message) in self.inbox.iter() {
            self.send(id, message)?
        }
        Ok(())
    }

    fn send(&self, to: WorkerId, msg: Message) -> Result<(), SendError<Message>> {
        self.mailboxes[to.value].send(msg)
    }
}

/// A single paxos worker.
pub struct Worker {
    /// The id of this worker.
    id: WorkerId,
    /// Number of workers in the cluster.
    nworkers: usize,
    /// Channel for receiving incoming messages.
    inbox: Receiver<Message>,
    /// Channel for sending outgoing messages.
    outbox: Sender<(WorkerId, Message)>,
    /// Persistent state.
    state: WorkerState,
    /// In-flight messages to track.
    in_flight: HashMap<RequestNumber, Vec<bool>>,
}

impl Worker {
    pub fn new(
        id: WorkerId,
        nworkers: usize,
        inbox: Receiver<Message>,
        outbox: Sender<(WorkerId, Message)>,
    ) -> Self {
        Self {
            id,
            nworkers,
            inbox,
            outbox,
            state: WorkerState::empty(),
            in_flight: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        for message in self.inbox.iter() {
            match message {
                Message::ExternalRequest(value) => {
                    match self.state.current_proposal {
                        // If we already have a proposal value, ignore new incoming requests.
                        Some(_) => (),
                        None => {
                            self.state.current_proposal = Some(Proposal {request_number: }
                        }
                    }
                }
                Message::Prepare(request) => (),
                Message::Prepared(response) => (),
                Message::Accept(request) => (),
                Message::Accepted(response) => (),
            }
        }
    }
}

pub struct Simulation {
    workers: Vec<JoinHandle<()>>,
    bus: MessageBus,
}

impl Simulation {
    pub fn new(nworkers: usize) -> Self {
        let mut handles = Vec::with_capacity(nworkers);
        let mut bus = MessageBus::new();

        for _ in 0..nworkers {
            let (id, outbox, inbox) = bus.add_slot();

            handles.push(spawn(move || {
                let mut worker = Worker::new(id, nworkers, inbox, outbox);
                worker.run()
            }))
        }

        Simulation {
            workers: handles,
            bus,
        }
    }
}

use crate::message::Message;
use crate::request::Request;
use crate::response::Response;
use crate::serialize::Serialize;

pub trait Target {
    type Inbound: Message;
    type Outbound: Message + Serialize;
}

#[derive(Debug)]
pub struct AsClient;

impl Target for AsClient {
    type Inbound = Response;
    type Outbound = Request;
}

#[derive(Debug)]
pub struct AsServer;

impl Target for AsServer {
    type Inbound = Request;
    type Outbound = Response;
}

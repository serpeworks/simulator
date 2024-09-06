use crate::io::{SerpeDialectReceiver, SerpeDialectSender};
use bevy::prelude::*;
use mavio::prelude::V2;
use mavio::{Endpoint, Receiver, Sender};
use std::net::TcpStream;

pub type BaseReceiver = Receiver<TcpStream, V2>;
pub type BaseSender = Sender<TcpStream, V2>;
pub type BaseEndpoint = Endpoint<V2>;

#[derive(Debug, Component)]
pub struct Connection {
    pub system_id: u8,
    pub receiver: SerpeDialectReceiver,
    pub sender: SerpeDialectSender,
}

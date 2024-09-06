use bevy::prelude::*;
use mavio::{prelude::V2, AsyncReceiver, AsyncSender, Endpoint, Frame, MavLinkId};
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{connection::Connection, coordinates::Coordinates},
    mavlink::dialects::{serpe_dialect::messages::Register, SerpeDialect},
};

pub enum IOMessage {
    CreateConnection {
        agent_id: u32,
        tx: tokio::sync::oneshot::Sender<Connection>,
        coordinates: Coordinates,
    },
}

pub type IOMessageReceiver = tokio::sync::mpsc::Receiver<IOMessage>;
pub type IOMessageSender = tokio::sync::mpsc::Sender<IOMessage>;
pub type SerpeDialectReceiver = tokio::sync::mpsc::Receiver<SerpeDialect>;
pub type SerpeDialectSender = tokio::sync::mpsc::Sender<SerpeDialect>;

pub type RealSender = AsyncSender<OwnedWriteHalf, V2>;
pub type RealReceiver = AsyncReceiver<OwnedReadHalf, V2>;

#[derive(Resource)]
pub struct IOResource {
    pub sender: IOMessageSender,
}

pub async fn send_registration(
    agent_id: u32,
    real_sender: &mut RealSender,
    coordinates: &Coordinates,
) -> Result<(), ()> {
    let message = Register {
        agent_id,
        latitude: coordinates.latitude,
        longitude: coordinates.longitude,
    };
    let first_frame = Frame::builder()
        .sequence(0)
        .system_id(0)
        .component_id(0)
        .version(V2)
        .message(&message)
        .map_err(|_| ())?
        .build()
        .into();

    real_sender.send(&first_frame).await.map_err(|_| ())?;
    Ok(())
}

pub async fn wait_for_register_ack(real_receiver: &mut RealReceiver) -> Result<u8, ()> {
    let first_frame = real_receiver.recv().await.map_err(|_| ())?;

    if let Ok(SerpeDialect::RegisterAck(msg)) = first_frame.decode::<SerpeDialect>() {
        Ok(msg.system_id)
    } else {
        Err(())
    }
}

pub async fn run_io(mut receiver: IOMessageReceiver, token: CancellationToken) {
    loop {
        select! {
            // Listen for the cancellation signal
            _ = token.cancelled() => {
                break;
            },

            // Listen for messages from the receiver
            maybe_message = receiver.recv() => {
                match maybe_message {
                    Some(IOMessage::CreateConnection {
                        agent_id,
                        tx,
                        coordinates,
                    }) => {
                        tokio::spawn(handle_new_connection(agent_id, tx, coordinates));
                    },
                    None => {
                        // If the receiver is closed, exit the loop
                        break;
                    }
                }
            },
        }
    }
}

async fn handle_new_connection(
    agent_id: u32,
    tx: tokio::sync::oneshot::Sender<Connection>,
    coordinates: Coordinates,
) {
    if let Ok(stream) = TcpStream::connect("127.0.0.1:8000").await {
        let (reader, writer) = stream.into_split();

        let mut real_sender = AsyncSender::versioned(writer, V2);
        let mut real_receiver = AsyncReceiver::versioned(reader, V2);

        if send_registration(agent_id, &mut real_sender, &coordinates)
            .await
            .is_err()
        {
            return;
        }

        // Save the system_id received from the register ack
        let system_id = match wait_for_register_ack(&mut real_receiver).await {
            Ok(id) => id,
            Err(_) => return, // Handle error appropriately
        };

        // Continue with your logic
        let (incoming_sender, incoming_receiver) = tokio::sync::mpsc::channel(256);
        let (outgoing_sender, outgoing_receiver) = tokio::sync::mpsc::channel(256);

        tx.send(Connection {
            system_id,
            receiver: incoming_receiver,
            sender: outgoing_sender,
        })
        .unwrap();

        let write_handle = tokio::spawn(write(outgoing_receiver, real_sender, system_id));
        let listen_handle = tokio::spawn(listen(incoming_sender, real_receiver));

        let _ = tokio::join!(listen_handle, write_handle);
    }
}

pub async fn write(
    mut outgoing_receiver: SerpeDialectReceiver,
    mut real_sender: RealSender,
    system_id: u8,
) {
    let endpoint = Endpoint::v2(MavLinkId::new(system_id, 0));
    while let Some(msg) = outgoing_receiver.recv().await {
        let frame = match msg {
            SerpeDialect::Register(msg) => endpoint.next_frame(&msg).unwrap(),
            SerpeDialect::Unregister(msg) => endpoint.next_frame(&msg).unwrap(),
            SerpeDialect::Heartbeat(msg) => endpoint.next_frame(&msg).unwrap(),
            SerpeDialect::MissionAccept(msg) => endpoint.next_frame(&msg).unwrap(),
            SerpeDialect::MissionUpdate(msg) => endpoint.next_frame(&msg).unwrap(),
            SerpeDialect::MissionFinished(msg) => endpoint.next_frame(&msg).unwrap(),
            _ => {
                continue;
            }
        };

        if let Err(_) = real_sender.send(&frame).await {
            println!("Error sending frame!");
            break;
        }
    }
}

pub async fn listen(sender: SerpeDialectSender, mut real_receiver: RealReceiver) {
    while let Ok(frame) = real_receiver.recv().await {
        match frame.decode::<SerpeDialect>() {
            Ok(message) => {
                match message {
                    SerpeDialect::HeartbeatAck(_) => {
                        // ignore hearbeat ack
                    }
                    SerpeDialect::MissionAcceptAck(_) => {
                        let _ = sender.try_send(message);
                    }
                    SerpeDialect::MissionRequest(_) => {
                        let _ = sender.try_send(message);
                    }
                    SerpeDialect::MissionFinishedAck(_) => {
                        let _ = sender.try_send(message);
                    }
                    _ => {
                        continue;
                    }
                }
            }
            Err(_) => {}
        }
    }
}

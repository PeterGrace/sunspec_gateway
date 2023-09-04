use std::future::Future;
use std::task::{Context, Poll};
use std::time::Duration;
use rumqttc::{ClientError, ConnectionError, Event, Incoming, Outgoing, QoS};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;
use crate::mqtt_connection::MqttConnection;
use crate::ipc::IPCMessage;

pub async fn mqtt_poll_loop(mqtt: MqttConnection, mut rx: Receiver<IPCMessage>) {
    let mut conn = mqtt.event_loop;
    loop {
        //region MQTT loop channel handling
        match rx.try_recv() {
            Ok(ipcm) => {
                match ipcm {
                    IPCMessage::Outbound(msg) => {
                        let payload = match serde_json::to_vec(&msg.payload) {
                            Ok(p) => p,
                            Err(e) => {
                                error!("Payload couldn't be serialized to vec: {e}");
                                continue;
                            }
                        };
                        if let Err(e) = mqtt.client.publish(msg.topic,
                                                            QoS::AtLeastOnce,
                                                            false,
                                                            payload,
                        ).await {
                            error!("Couldn't send message: {e}");
                        }
                    }
                    IPCMessage::Error(e) => {
                        error!("serial={}: {}", e.serial_number, e.msg);
                        return;
                    }
                }
            }
            Err(e) => {
                match e {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => {
                        error!("We are disconnected!");
                    }
                }
            }
        }
//endregion

        let notification = match conn.poll().await {
            Ok(event) => event,
            Err(e) => {
                let msg = format!("Unable to poll mqtt: {e}");
                error!(msg);
                return;
            }
        };
        match notification {
            Event::Incoming(i) => {
                match i {
                    Incoming::Disconnect => {
                        // we should do something here.
                        error!("mqtt disconnect packet received.");
                        return;
                    }
                    _ => {
                        info!("mqtt incoming packet: {:#?}", i);
                    }
                }
            }
            Event::Outgoing(o) => {
                match o {
                    _ => {
                        info!("outgoing mqtt packet: {:#?}", o);
                    }
                }
            }
        }
        let _ = sleep(Duration::from_millis(500));
    }
}

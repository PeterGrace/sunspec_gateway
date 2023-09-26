use crate::ipc::IPCMessage;
use crate::mqtt_connection::MqttConnection;
use rumqttc::{Event, Incoming, Outgoing, QoS};
use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, timeout};

const MQTT_POLL_INTERVAL_SECS: u16 = 1_u16;

pub async fn mqtt_poll_loop(mqtt: MqttConnection, mut rx: Receiver<IPCMessage>) {
    let task = tokio::spawn(async {
        let mut conn = mqtt.event_loop;
        let mut dlq: Vec<u16> = vec![];
        loop {
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
                        Incoming::ConnAck(ca) => {
                            info!("MQTT connection established.");
                        }
                        Incoming::PubAck(pa) => {
                            dlq.retain(|x| *x != pa.pkid);
                        }
                        Incoming::PingResp => {
                            trace!("Recv MQTT PONG");
                        }
                        Incoming::SubAck(_) => {}
                        Incoming::PubRec(pr) => {
                            info!("Received pubrec: {:#?}", pr);
                        }
                        _ => {
                            info!("mqtt incoming packet: {:#?}", i);
                        }
                    }
                }
                Event::Outgoing(o) => match o {
                    Outgoing::PingReq => {
                        trace!("Sent MQTT PING");
                    }
                    Outgoing::Publish(pb) => {
                        dlq.push(pb);
                    }
                    Outgoing::Subscribe(_) => {}
                    _ => {
                        info!("outgoing mqtt packet: {:#?}", o);
                    }
                },
            }
            if dlq.len() > 0 {
                trace!("DLQ is {}", dlq.len());
            }
        }
    });

    loop {
        //region MQTT loop channel handling
        match rx.try_recv() {
            Ok(ipcm) => match ipcm {
                IPCMessage::Outbound(msg) => {
                    let payload = match serde_json::to_vec(&msg.payload) {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Payload couldn't be serialized to vec: {e}");
                            continue;
                        }
                    };
                    match timeout(
                        Duration::from_secs(3),
                        mqtt.client
                            .publish(msg.topic, QoS::AtLeastOnce, false, payload),
                    )
                    .await
                    {
                        Ok(result) => match result {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Couldn't send message: {e}");
                            }
                        },
                        Err(e) => {
                            error!("Timeout trying to mqtt publish!")
                        }
                    }
                }
                IPCMessage::Error(e) => {
                    error!("serial={}: {}", e.serial_number, e.msg);
                    return;
                }
                IPCMessage::PleaseReconnect(_, _) => {
                    unreachable!();
                }
            },
            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {
                    error!("We are disconnected!");
                }
            },
        }

        //endregion
        // trace!("mqtt tick");
        let _ = sleep(Duration::from_millis(100)).await;
    }
}

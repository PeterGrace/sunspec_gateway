use anyhow;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use tokio::time::Duration;

const MQTT_KEEPALIVE_TIME: u64 = 5_u64;
const MQTT_THREAD_CHANNEL_CAPACITY: usize = 10_usize;
const MQTT_INBOUND_CONTROL_TOPIC: &str = "sunspec_gateway/input/#";

#[derive(Debug)]
pub struct MqttConnection {
    client_name: String,
    server_addr: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    pub(crate) client: AsyncClient,
    pub(crate) event_loop: MyEventLoop,
}

pub(crate) struct MyEventLoop(EventLoop);

impl Deref for MyEventLoop {
    type Target = EventLoop;

    //Target = EventLoop;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MyEventLoop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Debug for MyEventLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventLoop has no Debug.")
    }
}

impl MqttConnection {
    #[instrument]
    pub async fn new(
        client: String,
        addr: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> anyhow::Result<Self> {
        let mut mqttoptions = MqttOptions::new(&client, &addr, port);
        mqttoptions.set_keep_alive(Duration::from_secs(MQTT_KEEPALIVE_TIME));
        if username.is_some() && password.is_some() {
            mqttoptions.set_credentials(username.clone().unwrap(), password.clone().unwrap());
        }
        let (mut mqtt_client, mut eventloop) =
            AsyncClient::new(mqttoptions, MQTT_THREAD_CHANNEL_CAPACITY);

        match mqtt_client
            .subscribe(MQTT_INBOUND_CONTROL_TOPIC, QoS::AtMostOnce)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("Can't subscribe to inbound control topic: {e}");
            }
        }
        Ok(MqttConnection {
            client_name: client,
            server_addr: addr,
            port,
            username,
            password,
            client: mqtt_client,
            event_loop: MyEventLoop(eventloop),
        })
    }
}

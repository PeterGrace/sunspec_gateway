use anyhow;
use rumqttc::{AsyncClient, EventLoop, MqttOptions};
use tokio::time::Duration;

const MQTT_KEEPALIVE_TIME: u64 = 5_u64;
const MQTT_THREAD_CHANNEL_CAPACITY: usize = 10_usize;

pub struct MqttConnection {
    client_name: String,
    server_addr: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    pub(crate) client: AsyncClient,
    pub(crate) event_loop: EventLoop,
}

impl MqttConnection {
    pub fn new(
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

        Ok(MqttConnection {
            client_name: client,
            server_addr: addr,
            port,
            username,
            password,
            client: mqtt_client,
            event_loop: eventloop,
        })
    }
}

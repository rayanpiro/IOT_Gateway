use std::sync::Arc;

use super::get_mqtt_config;
use crate::device_protocols::DeviceProtocols;
use crate::models::tag::TagValue;
use crate::{gen_matcher, gen_readable_struct};
use gmqtt_client::{Message, MqttClient, MqttClientBuilder, QoS};
use serde_json;
use url::Url;

gen_matcher!(
    enum MqttQoS {
        AtMostOnce,
        AtLeastOnce,
        ExactlyOnce,
    }
);

impl MqttQoS {
    fn to_library_qos(&self) -> QoS {
        match self {
            MqttQoS::AtMostOnce => QoS::AtMostOnce,
            MqttQoS::AtLeastOnce => QoS::AtLeastOnce,
            MqttQoS::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

gen_matcher!(
    enum MqttProtocol {
        TCP,
        UDP,
    }
);

impl std::fmt::Display for MqttProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            MqttProtocol::TCP => "tcp",
            MqttProtocol::UDP => "udp",
        };
        write!(f, "{}", text)
    }
}

gen_readable_struct!(
    struct MqttIniConfig {
        protocol: MqttProtocol,
        host: String,
        port: u32,
        qos: MqttQoS,
        mqtt_topic_installation_prefix: String,
    }
);

#[derive(Debug)]
pub struct MqttError(String);

impl std::error::Error for MqttError {}

impl std::fmt::Display for MqttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

async fn process_recv_mqtt_command(
    client: MqttClient,
    msg: Message,
    devices: Arc<Vec<DeviceProtocols>>,
) {
    let payload = msg.payload_str().into_owned();
    let recv_tag_name = msg.topic().rsplit('/').next().unwrap();

    let splitted_payload = payload.split(' ').collect::<Vec<&str>>();
    let dev = devices.iter().find(|d| d.tag_name() == recv_tag_name);

    if dev.is_none() {
        return;
    }

    let dev = dev.unwrap();
    let topic_to_sent = msg.topic().replace("/commands", "");

    match splitted_payload.as_slice() {
        ["PING"] => {
            let result = dev.read().await;
            if result.is_ok() {
                send_message(&client, &topic_to_sent, "PONG").unwrap();
            } else {
                send_message(&client, &topic_to_sent, "Error").unwrap();
            }
        }
        ["READ"] => {
            let result = dev.read().await;
            let json = serde_json::to_string(&result).unwrap();
            send_message(&client, &topic_to_sent, &json).unwrap();
        }
        ["WRITE", value] => {
            let t_value = TagValue::I32(value.parse().unwrap());
            let result = dev.write(t_value).await;
            let json = serde_json::to_string(&result).unwrap();
            send_message(&client, &topic_to_sent, &json).unwrap();
            println!("WRITE VALUE: {} COMMAND", value);
        }
        _ => {
            println!("Invalid Command!");
        }
    };
}

pub fn send_message(client: &MqttClient, topic: &str, msg: &str) -> Result<(), MqttError> {
    client
        .publish_json(topic, msg, false, QoS::AtLeastOnce, None)
        .map_err(|err| MqttError(err.to_string()))?;

    Ok(())
}

pub fn connect_broker_subscribing_to_commands(
    devices: Arc<Vec<DeviceProtocols>>,
) -> Result<(MqttClient, String), MqttError> {
    let mqtt_config = get_mqtt_config();

    let protocol = mqtt_config.protocol.to_string();
    let broker_address = format!("{}://{}:{}", protocol, mqtt_config.host, mqtt_config.port);
    let topic_subscribe = format!("{}/commands/#", mqtt_config.mqtt_topic_installation_prefix);
    let qos = mqtt_config.qos.to_library_qos();

    let url = Url::parse(&broker_address).map_err(|err| MqttError(err.to_string()))?;

    let (mqtt_client, mqtt_worker) = MqttClientBuilder::new(url)
        .subscribe(topic_subscribe, qos)
        .build();

    let callback_mqtt_client = mqtt_client.clone();
    mqtt_client.set_on_message_callback(move |msg: &Message| {
        let msg_owned = msg.clone();
        tokio::spawn(process_recv_mqtt_command(
            callback_mqtt_client.clone(),
            msg_owned,
            devices.clone(),
        ));
    });

    tokio::spawn(mqtt_worker.run());

    Ok((
        mqtt_client,
        mqtt_config.mqtt_topic_installation_prefix,
    ))
}

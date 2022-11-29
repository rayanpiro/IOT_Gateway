use std::sync::Arc;

use crate::config_files::read_files::get_mqtt_config;
use crate::models::tag::{TTag, TagResponse, TagValue};
use crate::{gen_matcher, gen_readable_struct};
use gmqtt_client::{Message, MqttClient, MqttClientBuilder, QoS};
use url::Url;
use serde_json;

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

async fn process_recv_mqtt_command(client: MqttClient, msg: Message, tags: Vec<Arc<dyn TTag>>) {
    let payload = msg.payload_str().into_owned();
    let recv_tag_name = msg.topic()
        .rsplit("/")
        .collect::<Vec<&str>>()
        .into_iter()
        .take(2)
        .rev()
        .collect::<Vec<&str>>()
        .join("/");
    
    let splitted_payload = payload.split(" ").collect::<Vec<&str>>();
    let tag = tags.into_iter().find(|t| {
        let t_name = format!("{}/{}", t.get_device_name(), t.get_tag().get_name());
        t_name == recv_tag_name
    });

    if tag.is_none() {
        return;
    }

    let tag = tag.unwrap();
    let topic_to_sent = msg.topic().replace("/commands", "");

    match splitted_payload.as_slice() {
        ["PING"] => {
            let result = tag.read().await;
            if result.is_ok() {
                send_message(&client, &topic_to_sent, "PONG").unwrap();
            } else {
                send_message(&client, &topic_to_sent, "Error").unwrap();
            }
        },
        ["READ"] => {
            let result = tag.read().await;
            let json = serde_json::to_string(&result).unwrap();
            send_message(&client, &topic_to_sent, &json).unwrap();
        },
        ["WRITE", value] => {
            let t_value = TagValue::I32(i32::from_str_radix(value, 10).unwrap());
            let result = tag.write(t_value).await;
            let json = serde_json::to_string(&result).unwrap();
            send_message(&client, &topic_to_sent, &json).unwrap();
            println!("WRITE VALUE: {} COMMAND", value);
        },
        _ => {
            println!("Invalid Command!");
        },
    };
}

pub fn send_message(client: &MqttClient, topic: &str, msg: &str) -> Result<(), MqttError> {
    client
        .publish_json(topic, msg, false, QoS::AtLeastOnce, None)
        .map_err(|err| MqttError(err.to_string()))?;

    Ok(())
}

pub fn connect_broker_subscribing_to_commands(
    tags: Vec<Arc<dyn TTag>>,
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
            tokio::spawn(process_recv_mqtt_command(callback_mqtt_client.clone(), msg_owned, tags.clone()));
    });

    tokio::spawn(mqtt_worker.run());

    Ok((mqtt_client.clone(), mqtt_config.mqtt_topic_installation_prefix))
}

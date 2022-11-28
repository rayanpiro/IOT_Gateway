use std::str::Split;
use std::time::Instant;

use crate::config_files::read_files::get_mqtt_config;
use crate::models::tag::TagResponse;
use crate::{gen_matcher, gen_readable_struct};
use gmqtt_client::{MqttClient, MqttClientBuilder, QoS, Message};
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

fn process_recv_mqtt_command(msg: Message, _instant: Instant) {
    let payload = msg.payload_str().into_owned();
    let tag_name: &str = msg.topic()
        .split('/')
        .last()
        .unwrap();
    
    dbg!(tag_name);

    let splitted_payload = payload
        .split(" ")
        .collect::<Vec<&str>>();

    match splitted_payload.as_slice() {
        ["PING"] => {
            println!("PING COMMAND")
        },
        ["READ"] => {
            println!("READ COMMAND")
        },
        ["WRITE", value] => {
            println!("WRITE VALUE: {} COMMAND", value)
        },
        _ => {
            println!("Invalid Command!")
        },
    };
}

pub fn send_message(client: &MqttClient, topic: &str, msg: &TagResponse) -> Result<(), MqttError> {
    client
        .publish_json(topic, msg, false, QoS::AtLeastOnce, None)
        .map_err(|err| MqttError(err.to_string()))?;

    Ok(())
}

pub fn connect_broker_subscribing_to_commands() -> Result<(MqttClient, String), MqttError> {
    let mqtt_config = get_mqtt_config();

    let protocol = mqtt_config.protocol.to_string();
    let broker_address = format!("{}://{}:{}", protocol, mqtt_config.host, mqtt_config.port);
    let topic_subscribe = format!("{}/commands/#", mqtt_config.mqtt_topic_installation_prefix);
    let qos = mqtt_config.qos.to_library_qos();

    let url = Url::parse(&broker_address).map_err(|err| MqttError(err.to_string()))?;

    let (mqtt_client, mqtt_worker) = MqttClientBuilder::new(url)
        .on_message_owned_callback(process_recv_mqtt_command)
        .subscribe(topic_subscribe, qos)
        .build();

    tokio::spawn(mqtt_worker.run());

    Ok((mqtt_client, mqtt_config.mqtt_topic_installation_prefix))
}

pub mod mqtt;

use crate::config_files::ini_parser;
use mqtt::MqttIniConfig;

pub fn get_mqtt_config() -> MqttIniConfig {
    ini_parser::read_file::<MqttIniConfig>("mqtt.ini")
        .into_iter()
        .next()
        .expect("Invalid mqtt.ini file")
}

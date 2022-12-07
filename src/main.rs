mod cloud_protocols;
mod config_files;
mod device_protocols;
mod models;
mod running_modes;

use clap::Parser;
use cloud_protocols::mqtt::{connect_broker_subscribing_to_commands, send_message};
use device_protocols::DeviceProtocols;
use running_modes::{daemon_mode, tag_one_shot_read};
use std::sync::Arc;
use tokio;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    tag_name: Option<String>,

    #[arg(short, long, default_value_t = 1)]
    retry: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = Arc::new(DeviceProtocols::from_ini_files());
    let arguments = Args::parse();

    if let Some(tag_name) = arguments.tag_name {
        let return_value = tag_one_shot_read(devices, &tag_name, arguments.retry).await;
        print!("{}", return_value);
    } else {
        let (mqtt_client, base_topic) = connect_broker_subscribing_to_commands(devices.clone())
            .expect("There is a problem initializing Mqtt Conection");

        let sender = move |name: String, msg: String| {
            let topic = format!("{}/{}", base_topic, name);
            send_message(&mqtt_client, &topic, msg.as_ref())
        };
        daemon_mode(devices, sender).await;
    }
    Ok(())
}

use crate::cloud_protocols::mqtt::{connect_broker_subscribing_to_commands, send_message};
use crate::models::tag::TTag;
use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn daemon_mode(tags: Vec<Arc<dyn TTag>>) {
    let (mqtt_client, base_topic) = connect_broker_subscribing_to_commands()
        .expect("There is a problem initializing Mqtt Conection");
    
    let mqtt_client = Arc::new(mqtt_client);
    let base_topic = Arc::new(base_topic);

    let sched = JobScheduler::new().await.unwrap();
    for t in tags.iter() {

        let seconds = t.get_tag().get_freq().to_seconds();
        let t = t.clone();
        let mqtt_client = mqtt_client.clone();
        let base_topic = base_topic.clone();

        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_uuid, _l| {
            Box::pin({
                let t = t.clone();
                let mqtt_client = mqtt_client.clone();
                let base_topic = base_topic.clone();

                async move {
                    let topic = &format!("{}/{}/{}", &base_topic, t.get_device_name(), t.get_tag().get_name());
                    
                    if let Ok(value) = t.read().await {
                        send_message(&mqtt_client, topic, &value).unwrap();
                    }
                }
            })
        })
        .unwrap();
        sched.add(job).await.unwrap();
    }

    while sched.start().await.unwrap().await.is_err() {}
}

pub async fn tag_one_shot_read(
    tags: Vec<Arc<dyn TTag>>,
    tag_to_read: &str,
    retries: u32,
) -> String {
    let error_msg: String = "Error".to_string();

    let mut retries = retries;

    let tag = tags
        .iter()
        .find(|t| t.get_tag().get_name() == tag_to_read);

    if tag.is_none() {
        return error_msg;
    }

    while retries > 0 {
        if let Ok(x) = tag.unwrap().read().await {
                return x.value.to_string();
        }
        retries -= 1;
    }

    error_msg
}

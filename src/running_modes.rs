use crate::cloud_protocols::mqtt::MqttError;
use crate::device_protocols::Mode;
use crate::models::device::ReadError;
use crate::models::tag::TagResponse;
use crate::DeviceProtocols;
use futures::future::join_all;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

const TAG_REQUEST_SECONDS_TO_TIMEOUT: u64 = 4;

async fn job_function(tags_to_read: &Vec<DeviceProtocols>) -> String {
    let futures = tags_to_read.iter().map(|dev| {
        tokio::time::timeout(Duration::new(TAG_REQUEST_SECONDS_TO_TIMEOUT, 0), dev.read())
    });
    let values: Vec<Result<TagResponse, ReadError>> = join_all(futures)
        .await
        .into_iter()
        .map(|res| -> Result<TagResponse, ReadError> {
            match res {
                Ok(val) => match val {
                    Ok(val) => Ok(val),
                    Err(err) => Err(err),
                },
                Err(err) => Err(ReadError(err.to_string())),
            }
        })
        .collect();
    serde_json::to_string(&values).unwrap()
}

pub async fn daemon_mode<F>(devices: Arc<Vec<DeviceProtocols>>, send_f: F) -> !
where
    F: Fn(&str, &str) -> Result<(), MqttError> + Send + Sync + Clone + 'static,
{
    let set_of_connections: HashSet<String> =
        HashSet::from_iter(devices.iter().map(|d| d.device_name()));

    let sched = JobScheduler::new().await.unwrap();

    for connection_name in set_of_connections.iter() {
        let tags_to_read: Vec<DeviceProtocols> = devices
            .iter()
            .filter_map(|dev| {
                if dev.device_name() != *connection_name || dev.mode() != Mode::Read {
                    return None;
                }
                Some(dev.to_owned())
            })
            .collect();

        let first_device = tags_to_read.get(0).unwrap();
        let (seconds, device_name) = (first_device.freq().to_seconds(), first_device.device_name());
        let send_f = send_f.to_owned();

        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_uuid, _l| {
            let device_name = device_name.to_owned();
            let tags_to_read = tags_to_read.to_owned();
            let send_f = (&send_f).to_owned();
            Box::pin(async move {
                let json = job_function(&tags_to_read).await;
                send_f(&device_name, &json).unwrap();
            })
        });
        sched.add(job.unwrap()).await.unwrap();
    }
    loop {
        sched
            .start()
            .await
            .unwrap()
            .await
            .expect("There was an issue on the job sched.");
    }
}

pub async fn tag_one_shot_read(
    devices: Arc<Vec<DeviceProtocols>>,
    tag_to_read: &str,
    retries: u32,
) -> String {
    let error_msg: String = "Error".to_string();

    let device = devices.iter().find(|dev| dev.tag_name() == tag_to_read);

    if let Some(device) = device {
        let mut retries = retries;

        while retries > 0 {
            if let Ok(Ok(x)) = tokio::time::timeout(
                Duration::new(TAG_REQUEST_SECONDS_TO_TIMEOUT, 0),
                device.read(),
            )
            .await
            {
                return x.value.to_string();
            }
            retries -= 1;
        }
    }
    error_msg
}

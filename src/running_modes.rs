use crate::models::tag::TTag;
use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn daemon_mode(tags: Vec<Arc<dyn TTag>>) {
    let sched = JobScheduler::new().await.unwrap();

    for t in tags.iter() {
        let seconds = t.get_tag().get_freq().to_seconds();
        let t = t.clone();
        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_uuid, _l| {
            Box::pin({
                let t = t.clone();
                async move {
                    dbg!(t.get_tag().get_name());
                    if let Err(_) = dbg!(t.read().await) {
                        println!("Trying to read again tag {}!", t.get_tag().get_name());
                    }
                }
            })
        })
        .unwrap();
        sched.add(job).await.unwrap();
    }

    match sched.start().await.unwrap().await {
        Ok(_) => {}
        Err(_) => {}
    };
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
        .filter(|t| t.get_tag().get_name() == tag_to_read)
        .nth(0);

    if let None = tag {
        return error_msg;
    }

    while retries > 0 {
        match tag.unwrap().read().await {
            Ok(x) => {
                return x.value.to_string();
            }
            Err(_) => (),
        };
        retries -= 1;
    }
    return error_msg;
}

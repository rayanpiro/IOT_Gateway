mod ini_parser;
mod modbus_rtu;
mod modbus_tcp;
mod models;
use tokio;

const INI_PROTOCOL_FOLDERS: [&str; 2] = ["modbus_tcp/", "modbus_rtu/"];

use modbus_rtu::{ModbusRtuOverTCPConnection, ModbusRtuOverTCPDevice, ModbusRtuTag};
use modbus_tcp::{ModbusTcpConnection, ModbusTcpDevice, ModbusTcpTag};
use models::{
    device::THardDevice,
    tag::{TTag, TagId, TValidTag},
};
use std::{collections::HashMap, fmt::Debug, fs, marker::PhantomData, time::Duration};
use tokio_cron_scheduler::{JobScheduler, Job};

use std::sync::{Arc};
use tokio::sync::Mutex;
fn read_tags<T, C, S>(path: &str) -> Vec<Arc<dyn TTag>>
where
    T: THardDevice<C, S> + Clone + Send + Sync + 'static,
    C: TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <C as TryFrom<HashMap<String, String>>>::Error: Debug,
    S: TValidTag + TryFrom<HashMap<String, String>> + Debug + Clone + Send + Sync + 'static,
    <S as TryFrom<HashMap<String, String>>>::Error: Debug,
{
    let path = path.to_string();
    let connection = ini_parser::read_file::<C>(&(format!("{}/connection.ini", &path)))
        .into_iter()
        .nth(0)
        .unwrap();
    
    let mutex_device = Arc::new(Mutex::new(T::new(connection)));

    ini_parser::read_file::<S>(&(format!("{}/publishers.ini", &path)))
        .into_iter()
        .map(|t| {
            let tag: Arc<dyn TTag> = Arc::new(TagId {
                handler: Arc::clone(&mutex_device),
                tag: Arc::new(t),
                _phantom: PhantomData,
            });
            tag
        })
        .collect()
}

fn from_ini() -> Vec<Arc<dyn  TTag>> {
    let mut tags: Vec<Arc<dyn TTag>> = Vec::new();

    for folder in INI_PROTOCOL_FOLDERS {
        let protocol = &folder[..folder.len() - 1];

        for device in fs::read_dir(folder).unwrap() {
            let path = device.unwrap().path().to_str().unwrap().to_string();

            match protocol {
                "modbus_tcp" => tags.append(&mut read_tags::<
                    ModbusTcpDevice,
                    ModbusTcpConnection,
                    ModbusTcpTag,
                >(&path)),
                "modbus_rtu" => tags.append(&mut read_tags::<
                    ModbusRtuOverTCPDevice,
                    ModbusRtuOverTCPConnection,
                    ModbusRtuTag,
                >(&path)),
                _ => unimplemented!(),
            }
        }
    }

    tags
}

async fn daemon_mode(tags: Vec<Arc<dyn TTag>>) {
    let sched = JobScheduler::new().await.unwrap();

    for t in tags.iter() {
        let seconds = t.get_tag().get_freq().to_seconds();
        let t = t.clone();
        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_uuid, _l| Box::pin({
            let t = t.clone();
            async move {
                dbg!(t.get_tag().get_name());
                while let Err(_) = dbg!(t.read().await) {
                    println!("Trying to read again tag {}!", t.get_tag().get_name());
                }
            }
        })).unwrap();
        sched.add(job).await.unwrap();
    };

    dbg!(sched.start().await.unwrap().await);
}

async fn one_shot_read(tags: Vec<Arc<dyn TTag>>, tag_to_read: &str) {
    let filtered_tags = tags.iter()
        .filter(|t| t.get_tag().get_name() == tag_to_read);
        
    for t in filtered_tags {
        let t = t.clone();
        let res = t.read().await;
        match res {
            Ok(x) => print!("{}", x.value.to_string()),
            Err(_) => print!("Error"),
        };
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tags: Vec<Arc<dyn TTag>> = from_ini();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        one_shot_read(tags, args.get(1).unwrap()).await;
    }
    else {
        daemon_mode(tags).await;
    }
    
    Ok(())
}

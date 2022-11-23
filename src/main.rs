mod config_files;
mod device_protocols;
mod models;
mod running_modes;

use config_files::read_files::get_tags_from_ini_files;
use running_modes::{daemon_mode, tag_one_shot_read};
use tokio;

use clap::Parser;
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
    let tags = get_tags_from_ini_files();

    let arguments = Args::parse();

    if arguments.tag_name != None {
        let return_value =
            tag_one_shot_read(tags, &arguments.tag_name.unwrap(), arguments.retry).await;
        print!("{}", return_value);
    } else {
        daemon_mode(tags).await;
    }

    Ok(())
}

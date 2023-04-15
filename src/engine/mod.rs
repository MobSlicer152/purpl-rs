use crate::{platform, GAME_EXECUTABLE_NAME};
use chrono::Local;
use enum_iterator::Sequence;
use log::info;
use std::string::String;

#[derive(Debug, PartialEq, Sequence)]
pub enum DataDir {
    Root,
    Logs,
    Saves,
}

fn setup_logger() -> Result<(), fern::InitError> {
    let dt = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    fern::Dispatch::new()
        .format(|out, message, record| {
            let dt = Local::now();
            out.finish(format_args!(
                "[{} {} {}] {}",
                dt.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(
            get_data_dir(DataDir::Logs) + GAME_EXECUTABLE_NAME + "-" + &dt + ".log",
        )?)
        .apply()?;
    Ok(())
}

pub fn init() {
    for dir in enum_iterator::all::<DataDir>() {
        if std::fs::create_dir_all(get_data_dir(dir)).is_err() {
            panic!(
                "Failed to create engine data directory {}",
                get_data_dir(DataDir::Root)
            )
        }
    }

    if setup_logger().is_err() {
        panic!("Failed to set up logger");
    }

    info!("Engine initialization started");

    platform::video::init();
}

pub fn shutdown() {
    info!("Engine shutdown started");

    platform::video::shutdown();

    info!("Engine shutdown succeeded");
}

pub fn get_data_dir(subdir: DataDir) -> String {
    let basedirs = directories::BaseDirs::new().unwrap();
    let subdir_path = basedirs.data_dir();

    let path = String::from(subdir_path.to_str().unwrap())
        + "/"
        + crate::GAME_NAME
        + "/"
        + match subdir {
            DataDir::Root => "",
            DataDir::Logs => "logs/",
            DataDir::Saves => "saves/",
        };

    path.replace("\\", "/")
}

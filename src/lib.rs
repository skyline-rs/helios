#![feature(proc_macro_hygiene)]

use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use update_protocol::UpdateResponse;

pub fn get_helios_path() -> PathBuf {
    PathBuf::from(format!("sd:/helios/{:016X}", skyline::info::get_program_id()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub server_ip: IpAddr,
}

pub fn file_discovery() -> Result<Vec<(UpdateResponse, PathBuf)>, io::Error> {
    // Get Helios path for current Application
    let helios_path = get_helios_path();
    println!("[helios] Path to discover: {:?}", helios_path);

    // Does the directory exist?
    if !helios_path.exists() {
        fs::create_dir_all(&helios_path)?;
    }

    // Iterate through all the DirEntries
    fs::read_dir(helios_path)?
        .map(|entry| {
            let entry = entry?;

            //Ignore directories (for now?)
            if entry.file_type().unwrap().is_dir() {
                return Ok(None);
            }

            println!("About to read configuration");

            // Read the configuration
            let config: Config = open_config_toml(&entry.path()).unwrap();

            println!("Configuration read successfully");

            println!("About to ask server");
            // Ask the update server if an update is available and call our custom installer that will take care of updating the configuration
            
            let update = skyline_update::get_update_info(config.server_ip, &config.name, &config.version, false);

            println!("Finished asking server");

            Ok(update.map(|x| (x, entry.path())))
        })
        .filter_map(|x| x.transpose())
        .collect()
}


pub fn open_config_toml<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Option<Config>{
    println!("About to open configuration: {:#?}", path);

    match fs::read_to_string(path) {
        // The file was read successfully
        Ok(content) => {
            match toml::from_str(&content) {
                // Deserialized just fine
                Ok(conf) => Some(conf),
                // Couldn't deserialize the config
                Err(err) => {
                    println!("{}", err);
                    None
                },
            }
        },
        // Couldn't read the file to a String
        Err(err) => {
            println!("{}", err);
            None
        },
    }
}

pub fn update_config_toml<P: AsRef<Path>>(path: P, config: &Config) {
    println!("About to update configuration");

    // Convert the confix to a string
    let new_config = toml::to_string(&config).unwrap();
    // Write it back to the file
    fs::write(path, new_config);

    println!("Configuration updated successfully");

}

#[skyline::main(name = "helios")]
pub fn main() {
    match file_discovery() {
        Ok(_) => {},
        Err(error) => {
            println!("{}", error)
        },
    }

    // Should probably restart if mods were updated?
    //skyline::nn::oe::RestartProgramNoArgs();
}

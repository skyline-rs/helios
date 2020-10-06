#![feature(proc_macro_hygiene)]

use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use skyline_update::UpdateResponse;

fn get_helios_path() -> PathBuf {
    PathBuf::from(format!("sd:/helios/{:016X}", skyline::info::get_program_id()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    pub name: String,
    pub version: String,
    pub server_ip: IpAddr,
}

struct Update {
    response: UpdateResponse,
    config_path: PathBuf,
    ip: IpAddr,
    config: Config,
}

type Updates = Vec<Update>;

fn update_discovery() -> Result<Updates, io::Error> {
    // Get Helios path for current Application
    let helios_path = get_helios_path();
    println!("[helios] Path to discover: {:?}", helios_path);

    // Does the directory exist?
    if !helios_path.exists() {
        fs::create_dir_all(&helios_path)?;
    }

    // Iterate through all the DirEntries
    fs::read_dir(&helios_path)?
        .map(|entry| {
            let entry = entry?;

            //Ignore directories (for now?)
            if entry.file_type().unwrap().is_dir() {
                return Ok(None);
            }

            println!("About to read configuration");
            
            let config_path = helios_path.join(entry.path());

            // Read the configuration
            let config: Config = open_config_toml(&config_path).unwrap();
            let ip = config.server_ip;

            println!("Configuration read successfully");

            println!("About to ask server");
            // Ask the update server if an update is available and call our custom installer that will take care of updating the configuration
            
            let update = skyline_update::get_update_info(config.server_ip, &config.name, &config.version, false);

            println!("Finished asking server");


            Ok(update.map(|response| Update { response, config_path, ip, config } ))
        })
        .filter_map(|x| x.transpose())
        .collect()
}


fn open_config_toml<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Option<Config>{
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

fn update_config_toml<P: AsRef<Path>>(path: P, config: &Config) {
    println!("About to update configuration");

    // Convert the confix to a string
    let new_config = toml::to_string(&config).unwrap();
    // Write it back to the file
    fs::write(path, new_config);

    println!("Configuration updated successfully");

}

fn install(updates: &Updates) -> Result<(), ()> {
    for Update { response, ip, .. } in updates {
        skyline_update::install_update(*ip, response);
    }

    Ok(())
}

fn update_versions(updates: &Updates) -> Result<(), ()> {
    for Update { response, config_path, config, .. } in updates {
        let new_config = Config {
            version: response.new_plugin_version.clone(),
            ..config.clone()
        };
        update_config_toml(config_path, &new_config)
    }

    Ok(())
}

#[skyline::main(name = "helios")]
pub fn main() {
    let updates = update_discovery().unwrap();

    if updates.is_empty() {
        println!("[helios] No updates found");
        return;
    }

    let update_names = updates.iter().map(|Update { response, .. }| response.plugin_name.clone());

    let lines: Vec<String> = update_names.map(|name| format!("<li>{}</li>", name)).collect();
    
    let text = format!("Download the following updates?\n\n<ul style=\"max-height: 250px; overflow: hidden; overflow-y: scroll; text-align: left; display: inline-block;\">{}</ul>", lines.join("\n\n")); 

    if skyline_web::Dialog::yes_no(&text) {
        // Install updates
        install(&updates).unwrap();
        
        // Update versions
        update_versions(&updates).unwrap();

        // Restart
        skyline::nn::oe::RestartProgramNoArgs();
    }
}

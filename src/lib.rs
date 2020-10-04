#![feature(proc_macro_hygiene)]

use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::{ Path, PathBuf };

use serde::{ Deserialize, Serialize };

pub use update_protocol::UpdateResponse;

#[macro_export]
macro_rules! get_helios_path {
    () => {
        // Not too sure about that result
        //format!("sd:/helios/{}", skyline::info::get_program_id())
        "sd:/helios/01006A800016E000"
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub server_ip: IpAddr,
}

pub struct HeliosInstaller;

impl skyline_update::Installer for HeliosInstaller {
    fn should_update(&self, response: &UpdateResponse) -> bool {
        let result = skyline_web::Dialog::yes_no(format!(
            "An update for {} has been found.\n\nWould you like to download it?",
            response.plugin_name
        ));

        // If the user accepted the update
        if result {
            if let Some(new_version) = &response.new_plugin_version {
                // Get the configuration path using the plugin's name
                let config_path = format!("{}/{}.toml", get_helios_path!(), response.plugin_name);

                // Open the current configuration to edit it
                let mut config = open_config_toml(&config_path).unwrap();
                config.version = new_version.to_string();

                // Update the configuration
                update_config_toml(&config_path, &config);
            }
        }

        result
    }

    fn install_file(&self, path: PathBuf, buf: Vec<u8>) -> Result<(), ()> {
        let _ = std::fs::create_dir_all(path.parent().ok_or(())?);
        if let Err(e) = std::fs::write(path, buf) {
            println!("[updater] Error writing file to sd: {}", e);
            Err(())
        } else {
            Ok(())
        }
    }
}

pub fn file_discovery() -> Result<(), io::Error>{
    // Get Helios path for current Application
    let helios_path = PathBuf::from(get_helios_path!());
    println!("[helios] Path to discover: {:?}", helios_path);

    // Does the directory exist?
    if !helios_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "[helios] No directory found for this application"));
    }

    println!("Path exists");

    // Is it actually a file?
    // if helios_path.is_file() {
    //     return Err(io::Error::new(io::ErrorKind::Other, "[helios] Path refers to a file instead of a directory"));
    // }

    // Iterate through all the DirEntries
    for entry in fs::read_dir(helios_path)? {
        let entry = entry?;

        //Ignore directories (for now?)
        if entry.file_type().unwrap().is_dir() {
            continue;
        }

        println!("About to read configuration");

        // Read the configuration
        let config: Config = open_config_toml(&entry.path()).unwrap();

        println!("Configuration read successfully");

        println!("About to ask server");
        // Ask the update server if an update is available and call our custom installer that will take care of updating the configuration
        skyline_update::custom_check_update(config.server_ip, &config.name, &config.version, false, &HeliosInstaller);

        println!("Finished asking server");

    }

    Ok(())
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

use serde_derive::Deserialize;
use std::fs;
use std::io::Error;
use std::env;
use toml;

static CONFIG_FILE: &str = "./config/device_setup.toml";

#[derive(Deserialize)]
pub struct Data {
    pub devices: Vec<Devices>,
}

#[derive(Debug, Deserialize)]
pub struct Devices {
    pub name : String,
    pub address : String,
    pub pump_address : u16,
    pub flow_meter_address : u16,
    pub solenoid_output_address : u16,
    pub led_address : u16,
}

impl Default for Data {
    fn default() -> Data {
        Data {
            devices : Default::default()
        }
    }
}

impl Default for Devices {
    fn default() -> Devices {
        Devices {
            name : "Error".to_string(),
            address : "0.0.0.0".to_string(),
            pump_address : 999,
            flow_meter_address : 999,
            solenoid_output_address : 999,
            led_address: 999,
        }
    }
}

pub fn get_config() -> Result<Data, Error> {
    let contents = match fs::read_to_string(CONFIG_FILE) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", env::current_dir()?.display());
            panic!("{}", e)
        },
    };

    let data: Data = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(e) => {
            panic!("{}", e)
        },
    };

    Ok(data)
}
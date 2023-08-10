use rodbus::*;
use rodbus::client::*;

//use std::net::SocketAddr;
use std::time::Duration;
use std::str::FromStr;
use std::net::IpAddr;

/*#[tokio::main(flavor = "multi_thread")]*/
pub async fn read_input_coils(ip_address: &str, register_address: u16) -> Result<bool, RequestError> {

    let mut channel = spawn_tcp_client_task(
        HostAddr::ip(IpAddr::from_str(ip_address).unwrap(), 502),
        10,
        default_retry_strategy(),
        DecodeLevel::default(),
        None
    );

    let channel_result = channel.enable().await; 
    
    match channel_result {
        Ok(_) => debug!("Connected"),
        Err(e) => panic!("Could not connect {}", e)
    }

    let params = RequestParam::new(UnitId::new(1), Duration::from_secs(1));

    let result = channel
    .read_coils(params, AddressRange::try_from(register_address, register_address).unwrap())
    .await;

    match result {
        Ok(coils) => Ok(coils[0].value),
        Err(err) => Err(err),
    }
}

pub async fn read_input_register(ip_address: &str, register_address: u16) -> Result<u16, RequestError> {

    let mut channel = spawn_tcp_client_task(
        HostAddr::ip(IpAddr::from_str(ip_address).unwrap(), 502),
        10,
        default_retry_strategy(),
        DecodeLevel::default(),
        None
    );

    let channel_result = channel.enable().await; 
    
    match channel_result {
        Ok(_) => debug!("Connected"),
        Err(e) => panic!("Could not connect {}", e)
    }

    let params = RequestParam::new(UnitId::new(1), Duration::from_secs(1));

    let result = channel
    .read_input_registers(params, AddressRange::try_from(register_address, register_address).unwrap())
    .await;

    match result {
        Ok(coils) => Ok(coils[0].value),
        Err(err) => Err(err),
    }
}

pub async fn write_coil(ip_address: &str, register_address: u16, value: bool) -> Result<bool, RequestError> {

    let mut channel = spawn_tcp_client_task(
        HostAddr::ip(IpAddr::from_str(ip_address).unwrap(), 502),
        10,
        default_retry_strategy(),
        DecodeLevel::default(),
        None
    );

    let channel_result = channel.enable().await; 
    
    match channel_result {
        Ok(_) => debug!("Connected"),
        Err(e) => panic!("Could not connect {}", e)
    }

    let params = RequestParam::new(UnitId::new(1), Duration::from_secs(1));

    let result = channel
    .write_single_coil(params, Indexed::new(register_address, value))
    .await;

    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}
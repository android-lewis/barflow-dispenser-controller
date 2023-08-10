#[macro_use]
extern crate log;

use env_logger::Env;
use tokio;
use tonic::{transport::Server, Request, Response, Status};
use barflow::flow_service_server::{FlowService, FlowServiceServer};
use barflow::{BarflowRequest, BarflowResponse, FlowLevelRequest, FlowLevelResponse};
use std::time::Duration;
use std::time::Instant;
use std::thread::sleep;
use std::thread;

mod config_parser;
mod modbus_helper;
mod redis_connect;

pub mod barflow {
    tonic::include_proto!("barflow");
}

#[derive(Debug, Default)]
pub struct MyBarFlowService {}

pub fn dev_pour(request_message: BarflowRequest, controller_name: &str){
    //let request_message = request.into_inner();
    //Set dispensed value to 0
    match redis_connect::set_value(controller_name, 0.0) {
        Ok(_) => debug!("Value set in chache"),
        Err(e) => panic!("Could not set value in chache: {}", e)
    };
    let mut liquid_dispensed = 0.0;  
    // Get initial flowmeter value
    let zero_flowmeter_value = 18000.0;
    // Convert register value to voltage between 0-10v
    let zero_flowmeter_voltage = zero_flowmeter_value * (10.0/50000.0);

    // Loop until liquid dispensed is more than or equal to required volume
    while liquid_dispensed <= request_message.required_volume {
        // Set the measurement period and collect the current time
        let measurement_period = 1000;
        let initial_time = Instant::now();
        
        // Get initial flowmeter value and convert to voltage
        let initial_flowmeter_value = 18001.0;
        
        let initial_flowmeter_voltage = initial_flowmeter_value * (10.0/50000.0);
        
        // Sleep for measurment period
        sleep(Duration::from_millis(measurement_period));

        debug!("Initial time: {:?}", initial_time);
        debug!("Initial voltage: {:?}", initial_flowmeter_voltage);
        
        // Get final time and flowmeter value for the measurement period and convert to voltage
        let final_flowmeter_value = 25000.0;

        let final_flowmeter_voltage = final_flowmeter_value * (10.0/50000.0);
        let final_time = Instant::now();

        debug!("Final time: {:?}", final_time);
        debug!("Final voltage: {:?}", final_flowmeter_voltage);

        // Calculate the delta of time and voltage for our calulation on flow rate 
        let delta_time = final_time.duration_since(initial_time);
        let delta_voltage = match final_flowmeter_voltage - initial_flowmeter_voltage {
            0.0 => final_flowmeter_voltage - zero_flowmeter_voltage,
            _ => final_flowmeter_voltage - initial_flowmeter_voltage
        };

        debug!("Delta time: {:?}", delta_time);
        debug!("Delta voltage: {:?}", delta_voltage);

        // Q = (K*ΔV)/(Δt*60)
        let flowrate = (1420.0 * delta_voltage) as f32 / (delta_time.as_secs() * 60) as f32;
        
        liquid_dispensed += flowrate;
        match redis_connect::set_value(controller_name, liquid_dispensed) {
            Ok(_) => debug!("Value set in chache"),
            Err(e) => panic!("Could not set value in chache: {}", e)
        };
        debug!("Liquid dispensed: {:?}", liquid_dispensed);
    }
}

pub async fn pour(request_message: BarflowRequest, config_data: Vec<config_parser::helper::Devices>){
    //let request_message = request.into_inner();
    //Set dispensed value to 0
    
    
    let index = usize::try_from(request_message.controller_idx).unwrap();
    match redis_connect::set_value(&config_data[index].name, 0.0) {
        Ok(_) => debug!("Value set in chache"),
        Err(e) => panic!("Could not set value in chache: {}", e)
    };

    // Enable solenoid
    match modbus_helper::helper::write_coil(&config_data[index].address, config_data[index].solenoid_output_address, true).await {
        Ok(e) => debug!("Success, {}", e),
        Err(_) => panic!("Error writing coil"),
    };

    // Enable LED
    match modbus_helper::helper::write_coil(&config_data[index].address, config_data[index].led_address, true).await {
        Ok(e) => debug!("Success, {}", e),
        Err(_) => panic!("Error writing coil"),
    };
    
    let mut liquid_dispensed = 0.0;
    
    // Get initial flowmeter value
    let zero_flowmeter_value = match modbus_helper::helper::read_input_register(&config_data[index].address, config_data[index].flow_meter_address).await {
        Ok(e) => e as f32,
        Err(_) => panic!("Error reading coil"),
    };

    // Convert register value to voltage between 0-10v
    let zero_flowmeter_voltage = zero_flowmeter_value * (10.0/50000.0);

    // Loop until liquid dispensed is more than or equal to required volume
    while liquid_dispensed <= request_message.required_volume {
        // Set the measurement period and collect the current time
        let measurement_period = 1000;
        let initial_time = Instant::now();
        
        // Get initial flowmeter value and convert to voltage
        let initial_flowmeter_value = match modbus_helper::helper::read_input_register(&config_data[index].address, config_data[index].flow_meter_address).await {
            Ok(e) => e as f32,
            Err(_) => panic!("Error reading coil"),
        };
        
        let initial_flowmeter_voltage = initial_flowmeter_value * (10.0/50000.0);
        
        // Sleep for measurment period
        sleep(Duration::from_millis(measurement_period));

        debug!("Initial time: {:?}", initial_time);
        debug!("Initial voltage: {:?}", initial_flowmeter_voltage);
        
        // Get final time and flowmeter value for the measurement period and convert to voltage
        let final_flowmeter_value = match modbus_helper::helper::read_input_register(&config_data[index].address, config_data[index].flow_meter_address).await {
            Ok(e) => e as f32,
            Err(_) => panic!("Error reading coil"),
        };

        let final_flowmeter_voltage = final_flowmeter_value * (10.0/50000.0);
        let final_time = Instant::now();

        debug!("Final time: {:?}", final_time);
        debug!("Final voltage: {:?}", final_flowmeter_voltage);

        // Calculate the delta of time and voltage for our calulation on flow rate 
        let delta_time = final_time.duration_since(initial_time);
        let delta_voltage = match final_flowmeter_voltage - initial_flowmeter_voltage {
            0.0 => final_flowmeter_voltage - zero_flowmeter_voltage,
            _ => final_flowmeter_voltage - initial_flowmeter_voltage
        };

        debug!("Delta time: {:?}", delta_time);
        debug!("Delta voltage: {:?}", delta_voltage);

        // Q = (K*ΔV)/(Δt*60)
        let flowrate = (1420.0 * delta_voltage) as f32 / (delta_time.as_secs() * 60) as f32;
        
        liquid_dispensed += flowrate;
        match redis_connect::set_value(&config_data[index].name, liquid_dispensed) {
            Ok(_) => debug!("Value set in chache"),
            Err(e) => panic!("Could not set value in chache: {}", e)
        };
        debug!("Liquid dispensed: {:?}", liquid_dispensed);
    }

    // Disable solenoid from pump
    match modbus_helper::helper::write_coil(&config_data[index].address, config_data[index].solenoid_output_address, false).await {
        Ok(e) => debug!("Success, {}", e),
        Err(_) => panic!("Error writing coil"),
    };

    // Turn off LED
    match modbus_helper::helper::write_coil(&config_data[index].address, config_data[index].led_address, false).await {
        Ok(e) => debug!("Success, {}", e),
        Err(_) => panic!("Error writing coil"),
    };
}

#[tonic::async_trait]
impl FlowService for MyBarFlowService {
    async fn flow(&self,
        request: Request<BarflowRequest>) -> Result<Response<BarflowResponse>, Status> {
            debug!("Got a request from {:?}:{:?}", request.remote_addr().unwrap().ip(), request.remote_addr().unwrap().port());
            
            let request_message = request.into_inner();
            debug!("Got a message {:?}", request_message.controller_idx);
            debug!("Got a message {:?}", request_message.tap_idx);
            debug!("Got a message {:?}", request_message.register_address);
            debug!("Got a message {:?}", request_message.required_volume);
    
            // Parse remote address against list of devices from toml file *
            // Enable the solenoid for the device *
            // Enable Green led ^
            // Loop for feedback from flow sensor every second *
            // Once liquid level is reached, disable solenoid
            // Disable Green LED ^
            
            let config_data = match config_parser::helper::get_config(){
                Ok(s) => s.devices,
                Err(err) => panic!("{}", err),
            };
    
            let index = usize::try_from(request_message.controller_idx).unwrap();
            debug!("{:?}", config_data[index].address);

            //thread::spawn(move || { dev_pour(request_message, &config_data[index].name) });
            thread::spawn(move || { pour(request_message, config_data) });
    
            let reply = barflow::BarflowResponse {
                success: true,
            };
    
            Ok(Response::new(reply))
        }

    async fn flow_level(&self,
    request: Request<FlowLevelRequest>) -> Result<Response<FlowLevelResponse>, Status> {
        debug!("Got a request from {:?}:{:?}", request.remote_addr().unwrap().ip(), request.remote_addr().unwrap().port());
        
        let request_message = request.into_inner();
        debug!("Got a message {:?}", request_message.controller_idx);
        debug!("Got a message {:?}", request_message.tap_idx);

        let config_data = match config_parser::helper::get_config(){
            Ok(s) => s.devices,
            Err(err) => panic!("{}", err),
        };

        let index = usize::try_from(request_message.controller_idx).unwrap();

        let flow_level = match redis_connect::get_value(&config_data[index].name){
            Ok(lvl) => lvl,
            Err(e) => panic!("Could not fetch requested value: {}", e),
        };
        // Check persistent data store that holds the amount of volume dispensed for related tap
        let reply = barflow::FlowLevelResponse {
            volume_dispensed: flow_level,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let address = "[::0]:44144".parse().unwrap();
    let barflow_service = MyBarFlowService::default();
    let svc = FlowServiceServer::new(barflow_service);
    Server::builder().add_service(svc)
        .serve(address)
        .await?;
    Ok(())
}
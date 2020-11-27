#![allow(dead_code)]

mod media;
mod cast;

use std::{io::{self, Write}, thread};
use std::fs::File;

use cast::Caster;

#[tokio::main]
async fn main() {
    let media_file = String::from("test.mp4"); //select_file();
    println!("Hosting media.");
    let (_addr, _shutdown) = media::host_media(media_file).await.unwrap();
    println!("Finding chromecasts.");
    let device_ips = cast::find_device_ips().await.unwrap();
    let device_ip = device_ips.first().unwrap();
    
    println!("Starting cast.");
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();
    let (_handle, caster) = Caster::launch_media(&device_ip.to_string(), shutdown_rx).unwrap();
    
    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            input_tx.send(get_input(">")).unwrap();
        }
    });

    // Main thread loop
    let mut last_status = cast::MediaStatus::Inactive;
    loop{
        // Handle input events
        if let Ok(input) = input_rx.try_recv(){
            match &input[..] {
                "pause" => {caster.pause().unwrap();},
                "play" => {caster.resume().unwrap();},
                "stop" => {caster.stop().unwrap();},
                "seek" => {caster.seek(29.0).unwrap();},
                "kill" => {shutdown_tx.send(()).unwrap();},
                "status" => {println!("[Media Status] {:?}", last_status);}
                _ => {},
            }
        };
        
        // Handle media status events
        if let Ok(msg) = caster.status_rx.try_recv(){
            last_status = msg;

            // Check if the caster has stopped
            if let cast::MediaStatus::Inactive = &last_status {
                println!("Media inactive, shutting down.");
                shutdown_tx.send(()).unwrap();
                break;
            }
        }

        // Handle device status updates 
        if let Ok(_) = caster.device_rx.try_recv() {
            //println!("{}", msg);
        }
    }
}      

fn _select_file() -> String {
    println!("Select a file to cast:");
    let mut input = String::new();
    while input.is_empty() {
        input = get_input("> ");
        if input.len() > 4 && &input[input.len()-4..] == ".mp4" {
            if let Ok(_) = File::open(&input){
                return String::from(input);
            }else{
                println!("File not found.");
            }
        }
        else {
            println!("Only .mp4 files are supported.");
        }
        input = String::new();
    }
    
    return String::new();
}

fn get_input(prompt: &str) -> String {
    // Print the prompt immediately
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    // Read user input to a buffer
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    // Trim newline
    String::from(buffer.trim_end())
}

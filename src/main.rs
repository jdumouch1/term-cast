#![allow(dead_code)]

mod fileserver;
mod cast;

use std::io::{self, Write};
use std::fs::File;

use cast::Caster;

#[tokio::main]
async fn main() {
    let media_file = String::from("test.mp4"); //select_file();
    println!("Hosting media.");
    let (_addr, _shutdown) = fileserver::host_media(media_file).await.unwrap();
    println!("Finding chromecasts.");
    let device_ips = cast::find_device_ips().await.unwrap();
    let device_ip = device_ips.first().unwrap();
    
    println!("Starting cast.");
    let caster = Caster::spawn(device_ip.to_string()).unwrap();
    loop{
        match &get_input(">")[..] {
            "p" => {caster.pause().unwrap();},
            "l" => {caster.resume().unwrap();},
            _ => {},
        };
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

#![allow(dead_code, unused_variables)]
mod error;

use error::CastError;
use mdns::{Record, RecordKind};
use futures::future;
use futures_util::{pin_mut, stream::StreamExt};
use std::{
    sync::mpsc::{Receiver, TryRecvError}, 
    thread::{self, JoinHandle}, 
    time::{SystemTime, Duration},
    net::IpAddr,
};
use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::{
    heartbeat::HeartbeatResponse,
    media::{Media, StatusEntry, StreamType},
    receiver::CastDeviceApp,
};

const DESTINATION_ID: &'static str = "receiver-0";
const SERVICE_NAME: &'static str = "_googlecast._tcp.local";
const TIMEOUT_SECONDS: u64 = 3;

enum PlayerSignal {
    Play, // Resume
    Pause, 
    Stop,
    Seek(f32),        
}

#[derive(Debug)]
pub enum MediaStatus {
    Active(StatusEntry),
    Inactive,
}

pub struct Caster {
    device_addr: String,
    /// Receiver for logging device messages
    pub device_rx: Receiver<String>,
    /// Receiver for acquiring playback info
    pub status_rx: Receiver<MediaStatus>,
}

impl Caster {
    /// Connect to the chromecast at device_addr and launch the media
    /// playing app and media at "http://localhost:1544"      
    /// A thread is spawned that handles keep alive and playback info.  
    /// Both channels are assigned to the Caster that is returned.
    /// #### Arguments 
    /// - ```device_addr: &str``` The IP address of the Chromecast.
    /// - ```shutdown_rx: Receiver<()>``` A graceful shutdown channel.
    /// #### Returns 
    /// - ```JoinHandle<()>``` The join handle of the status thread.
    /// - ```Self``` A new Caster struct.
    pub fn launch_media(device_addr: &str, shutdown_rx: Receiver<()>) 
        -> Result<(JoinHandle<()>, Self), CastError> {
        // Create a string copy of device_addr to pass to the thread
        let addr = String::from(device_addr);

        // Channel for logging
        let (device_tx, device_rx) = std::sync::mpsc::channel::<String>();
        // Channel for playback info
        let (status_tx, status_rx) = std::sync::mpsc::channel::<MediaStatus>();

        let mut last_up = SystemTime::now();
        // Open a thread to handle recieve status updates
        let handle = thread::spawn(move || {
            // Open the device connection
            let device = CastDevice::
                connect_without_host_verification(addr, 8009).unwrap();
            device.connection.connect(DESTINATION_ID).unwrap();
            device.heartbeat.ping().unwrap();
    
            // Launch the media player on the device
            let app = device.receiver.launch_app(
                &CastDeviceApp::DefaultMediaReceiver).unwrap();
            let transport_id = app.transport_id.to_string();
            let session_id = app.session_id.to_string();
            
            // Connect to the app and begin playback
            device.connection.connect(&transport_id).unwrap();
            device.media.load(
                &transport_id, 
                &session_id, 
                &Media {
                    content_id: "http:/192.168.0.10:1544".to_string(),
                    content_type: "video/mp4".to_string(),
                    stream_type: StreamType::Buffered,
                    duration: None,
                    metadata: None,
                },
            ).unwrap();

            // Chromecast communication loop
            loop { 
                // Poll the shutdown reciever
                match shutdown_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        // Break thread loop
                        println!("Killing status communication thread.");
                        return;                        
                    },
                    Err(TryRecvError::Empty) => {}
                }

                // Send available log to main thread
                // Device status must be handled or else the 
                // chromecast will overflow on an unhandled ping
                if let Some(msg) = Caster::handle_device_status(&device){
                    if let Err(err) = device_tx.send(msg){
                        eprintln!("Failed to send message log to 
                                    main thread: {:?}", err);
                    }
                }

                // Gather media status at most once per second
                let millis_since_last = last_up.elapsed().unwrap().as_millis();
                if millis_since_last >= 1000 {
                    // Retrieve media status
                    let statuses = device.media
                        .get_status(&transport_id, None)
                        .unwrap();
                    // Map StatusEntry to MediaStatus enum
                    let status = match statuses.entries.first() {
                        Some(status) => MediaStatus::Active(status.clone()),
                        None => MediaStatus::Inactive
                    };
                    // Send to main thread
                    if let Err(err) = status_tx.send(status) {
                        eprintln!(
                            "Failed to send media status to main thread: {:?}",
                            err);
                    }
                    last_up = SystemTime::now();
                }
                
            }
        });
        // Create a string copy of device_addr to store on Caster struct
        let device_addr = String::from(device_addr);
        Ok((handle, Self {device_addr, device_rx, status_rx }))
    }


    /// Block until device status is recieved.  
    /// The message is parsed into a string, and returned.  
    /// If the message was a Heartbeat, a pong will be returned to the 
    /// chromecast.
    /// ### Returns
    /// - On success: ***Some(Log message as String)***
    /// - On error: ***None***
    fn handle_device_status(device: &CastDevice) -> Option<String> {
        match device.receive() {
            Ok(msg) => {
                let log_msg: String;
                match msg {
                    ChannelMessage::Connection(resp) => {
                        return Some(format!("[Device=>Connection] {:?}", 
                            resp));
                    }
                    ChannelMessage::Media(resp) => {
                        return Some(format!("[Device=>Media] {:?}", resp));
                    }
                    ChannelMessage::Receiver(resp) => {
                        return Some(format!("[Device=>Receiver] {:?}", 
                            resp));
                    }
                    ChannelMessage::Raw(resp) => {
                        return Some(format!("[Device] Message could not 
                                            be parsed: {:?}", resp));
                    }
                    ChannelMessage::Heartbeat(resp) => {
                        // Reply to ping with pong
                        if let HeartbeatResponse::Ping = resp {
                            device.heartbeat.pong().unwrap();
                        }
                        return Some(format!("[Heartbeat] {:?}", resp));
                    }
                }
            },
            // Failed to recieve message
            Err(err) => {
                eprintln!("An error occured while recieving 
                            message from chromecast:\n{:?}", err);
                return None
            }
        }
    }

    /// Resumes playback on chromecast if it is paused.
    pub fn resume(&self) -> Result<(), CastError> {
        self.change_media_state(PlayerSignal::Play)?;
        Ok(())
    }
    
    /// Pauses playback on chromecast if it is playing.
    pub fn pause(&self) -> Result<(), CastError> {
        self.change_media_state(PlayerSignal::Pause)?;
        Ok(())
    }
    
    /// Stops playback and returns to the splashscreen
    pub fn stop(&self) -> Result<(), CastError> {
        self.change_media_state(PlayerSignal::Stop)?;
        Ok(())
    }

    /// Seek current playback to specified time.
    /// ### Arguments 
    /// * time - A float representing the time in seconds to
    ///     seek to.
    pub fn seek(&self, time: f32) -> Result<(), CastError> {
        self.change_media_state(PlayerSignal::Seek(time))?;
        Ok(())
    }

    /// Calls one of the functions that alter the play state
    /// on the current playback. 
    /// ### Arguments
    /// * state - A MediaState to apply to the current playback
    fn change_media_state(&self, state: PlayerSignal) -> Result<(),CastError> {
        // Open a new connection
        let device = self.connect()?;
        let status = device.receiver.get_status()?;
        let app = status.applications.first().unwrap();

        // Connect to application
        device.connection.connect(app.transport_id.to_string())?;

        let media_status = device.media
            .get_status(
                app.transport_id.as_str(), 
                None)?;

        // Ensure that media_status has an entry and take the first
        if let Some(media_status) = media_status.entries.first(){
            let transport_id = app.transport_id.as_str();
            let session_id = media_status.media_session_id;

            // Signal the state to the chromecast
            match state {
                PlayerSignal::Play => {
                    device.media.play(transport_id, session_id)?;
                }
                PlayerSignal::Pause => {
                    device.media.pause(transport_id, session_id)?;
                }
                PlayerSignal::Stop => {
                    device.media.stop(transport_id, session_id)?;
                }
                PlayerSignal::Seek(time) => {
                    device.media.seek(
                        transport_id, session_id,
                        Some(time),     // Time to seek to
                        None)?;         // Resume State (leave state unchanged)
                }
            }
        }else{
            return Err(CastError::CasterError(
                "Cannot change media state. No active media."));
        }
        device.connection.disconnect(DESTINATION_ID).unwrap();
        Ok(())
    }

    /// Create a new CastDevice connection.  
    /// *Note: This connection must either be kept-alive with ping/pong 
    /// or closed after a short period of time.*
    fn connect(&self) -> Result<CastDevice, CastError> {
        let device = match CastDevice::connect_without_host_verification(
            &self.device_addr, 
            8009){
                
            Ok(device) => device,
            Err(err) => {
                panic!("Failed to establish connection to device: {:?}", err);
            }
        };
        device.connection.connect(DESTINATION_ID).unwrap();
        Ok(device)
    }
}

/// Scan DNS records matching chromecast service names.  
/// ### Returns
/// Vec\<IpAddr\> - A Result containing a list of chromecast
///     IP addresses discovered on the network. 
pub async fn find_device_ips() -> Result<Vec<IpAddr>, mdns::Error> {
    // Create timeout vars
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start_time = SystemTime::now();
    
    // Create the discovery stream
    let stream = mdns::discover::all(SERVICE_NAME, timeout)?
        .listen()
        .take_while(|_|future::ready(start_time.elapsed().unwrap() < timeout));
    pin_mut!(stream);

    // Listen and add devices to vec
    let mut device_ips = Vec::new();
    while let Some(Ok(resp)) = stream.next().await {
        let addr = resp.records()
            .filter_map(self::to_ip_addr)
            .next();
        if let Some(addr) = addr {
            if !device_ips.contains(&addr) {
                device_ips.push(addr.clone());
            }
        }
    }

    Ok(device_ips)
}

/// Convert a DNS record to IpAddr
/// ### Returns
/// ```Some<IpAddr>``` If record is A or AAAA  
/// Otherwise   
/// ```None```   
fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    //TODO: Match the record friendly name with IP address on record id
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

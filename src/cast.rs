#![allow(unused_imports)]
#![allow(dead_code, unused_variables)]

use warp::Filter;
use tokio::sync::{Mutex, broadcast};
use std::{
    net::UdpSocket, 
    sync::mpsc::Receiver, 
    thread, time::SystemTime
};
use rust_cast::{channels::receiver::Application, CastDevice, ChannelMessage};
use rust_cast::channels::{
    heartbeat::HeartbeatResponse,
    media::{Media, StatusEntry, StreamType},
    receiver::CastDeviceApp,
};
use mdns::{Record, RecordKind};
use futures::{FutureExt, future};
use futures_util::{pin_mut, stream::StreamExt};
use std::{net::IpAddr, time::{Duration,Instant}};

const DESTINATION_ID: &'static str = "receiver-0";
const SERVICE_NAME: &'static str = "_googlecast._tcp.local";
const TIMEOUT_SECONDS: u64 = 3;

#[derive(Clone)]
pub enum CastEvent {
    Play(String),
    Pause,
    Stop,
    Kill,
}
pub enum CastResponse {
    Error
}

#[derive(Debug)]
pub enum CastError {
    CastError(rust_cast::errors::Error),
    IoError(std::io::Error),
    ServerError,
    CasterError(&'static str),
}

impl From<rust_cast::errors::Error> for CastError {
    fn from(err: rust_cast::errors::Error) -> Self {
        CastError::CastError(err)
    }
}
impl From<std::io::Error> for CastError {
    fn from(err: std::io::Error) -> Self {
        CastError::IoError(err)
    }
}

enum MediaState {
    Play,
    Pause,
    Stop,
    Seek(f32),        
}

pub struct Caster {
    cast_addr: String,
    status_rx: Receiver<StatusEntry>,
}



impl Caster {
    pub fn spawn(cast_addr: String) -> Result<Self, CastError> {
        let addr = cast_addr.clone();
        let mut last_iter = SystemTime::now();
        let (tx, rx) = std::sync::mpsc::channel::<StatusEntry>();
        thread::spawn(move || {
            let device = CastDevice::connect_without_host_verification(addr, 8009).unwrap();
            device.connection.connect(DESTINATION_ID).unwrap();
            device.heartbeat.ping().unwrap();
    
            let app = device.receiver.launch_app(
                &CastDeviceApp::DefaultMediaReceiver).unwrap();
            
            device.connection.connect(app.transport_id.to_string()).unwrap();
            let transport_id = app.transport_id.to_string();
            let session_id = app.session_id.to_string();

            // Begin playback
            device.media.load(
                app.transport_id.to_string(), 
                app.session_id.to_string(), 
                &Media {
                    content_id: "http:/192.168.0.10:1544".to_string(),
                    content_type: "video/mp4".to_string(),
                    stream_type: StreamType::Buffered,
                    duration: None,
                    metadata: None,
                },
            ).unwrap();

            loop { 
                match device.receive(){
                    Ok(ChannelMessage::Heartbeat(resp)) => {
                        println!("[Heartbeat] {:?}", resp);
    
                        if let HeartbeatResponse::Ping = resp {
                            device.heartbeat.pong().unwrap();
                        }
                    }
                    Ok(ChannelMessage::Connection(resp)) => {}
                    Ok(ChannelMessage::Media(resp)) => {}
                    Ok(ChannelMessage::Receiver(resp)) => {}
                    Ok(ChannelMessage::Raw(resp)) => {}
                    Err(err) => { 
                        eprintln!("An error occured while receiving 
                                   message from chromecast:\n{:?}", err); 
                    }
                }

                let elapsed = last_iter.elapsed().unwrap().as_millis();
                if elapsed >= 1000 {
                    // Retrieve media status
                    let statuses = device.media
                        .get_status(&transport_id, None)
                        .unwrap();
                    let status = statuses.entries.first().unwrap();
                    // Send to main thread
                    if let Err(err) = tx.send(status.clone()) {
                        eprintln!(
                            "Failed to send media status to main thread: {:?}",
                            err);
                    }
                    // Re-up the timer
                    last_iter = SystemTime::now();
                }
                
            }
        });
        Ok(Self { cast_addr, status_rx: rx })
    }

    pub fn begin_playback(&self) -> Result<(), CastError> {
        // Open a new connection
        let device = self.connect()?;
        let status = device.receiver.get_status()?;
        let app = match status.applications.first() {
            Some(app) => app,
            None => return Err(CastError::CasterError("No application is running on the chromecast.")),
        };

        device.connection.connect(app.transport_id.to_string())?;
        
        // Begin playback
        device.media.load(
            app.transport_id.to_string(), 
            app.session_id.to_string(), 
            &Media {
                content_id: "http:/192.168.0.10:1544".to_string(),
                content_type: "video/mp4".to_string(),
                stream_type: StreamType::Buffered,
                duration: None,
                metadata: None,
            },
        )?;

        // Close the connection again
        device.connection.disconnect(DESTINATION_ID)?;

        Ok(())
    }

    /// Resumes playback on chromecast if it is paused.
    pub fn resume(&self) -> Result<(), CastError> {
        self.change_media_state(MediaState::Play)?;
        Ok(())
    }
    
    /// Pauses playback on chromecast if it is playing.
    pub fn pause(&self) -> Result<(), CastError> {
        self.change_media_state(MediaState::Pause)?;
        Ok(())
    }
    
    /// Stops playback and returns to the splashscreen
    pub fn stop(&self) -> Result<(), CastError> {
        self.change_media_state(MediaState::Stop)?;
        Ok(())
    }

    /// Seek current playback to specified time.
    /// # Arguments 
    /// * time - A float representing the time in seconds to
    ///     seek to.
    pub fn seek(&self, time: f32) -> Result<(), CastError> {
        self.change_media_state(MediaState::Seek(time))?;
        Ok(())
    }

    /// Calls one of the functions that alter the play state
    /// on the current playback. 
    /// ### Arguments
    /// * state - A MediaState to apply to the current playback
    fn change_media_state(&self, state: MediaState) -> Result<(), CastError> {
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
                MediaState::Play => {
                    device.media.play(transport_id, session_id)?;
                }
                MediaState::Pause => {
                    device.media.pause(transport_id, session_id)?;
                }
                MediaState::Stop => {
                    device.media.stop(transport_id, session_id)?;
                }
                MediaState::Seek(time) => {
                    device.media.seek(transport_id, session_id, Some(time), None)?;
                }
            }
        }else{
            return Err(CastError::CasterError(
                "Resume failed. 'media_status' has no entries."));
        }

        device.connection.disconnect(DESTINATION_ID).unwrap();

        Ok(())
    }

    fn connect(&self) -> Result<CastDevice, CastError> {
        let device = match CastDevice::connect_without_host_verification(
            &self.cast_addr, 
            8009){
                
            Ok(device) => device,
            Err(err) => {panic!("Failed to establish connection to device: {:?}", err);}
        };
        device.connection.connect(DESTINATION_ID).unwrap();
        Ok(device)
    }
}

/// Scan DNS records matching chromecast service names.  
/// # Returns
/// 'Vec\<IpAddr\>' - A Result containing a list of chromecast
///     IP addresses discovered on the network. 
pub async fn find_device_ips() -> Result<Vec<IpAddr>, mdns::Error> {
    // Create timeout vars
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start_time = Instant::now();
    
    // Create the discovery stream
    let stream = mdns::discover::all(SERVICE_NAME, timeout)?
        .listen()
        .take_while(|_| future::ready(start_time.elapsed() < timeout));
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

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    //TODO: Match the record friendly name with IP address using the id to match them
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}



/* Signal the chromecast to play media
device.media.load(
    app.transport_id.as_str(), 
    app.session_id.as_str(),
    &Media {
        content_id: String::from(&media_addr),
        content_type: "video/mp4".into(),
        stream_type: StreamType::Buffered,
        duration: None,
        metadata: None,
    })?;*/

#![allow(dead_code)]

use tokio::sync::oneshot;
use std::net::UdpSocket;

use std::process::Command;

pub async fn host_media(media: String) -> Result<(String, oneshot::Sender<()>), std::io::Error> {
    // Get local ip address
    let local_ip = get_local_ip()?;
    let media_addr = format!("http://{}:1544", local_ip);
        
    let (shutdown_sender, shutdown_reciever) = oneshot::channel::<()>(); 
    let route = warp::fs::file(media);
    let (_addr, server) = warp::serve(route)
        .bind_with_graceful_shutdown(([0,0,0,0], 1544), async {
            shutdown_reciever.await.ok(); 
        });
    tokio::task::spawn(server);
    Ok((media_addr, shutdown_sender))
}

pub fn convert_to_mp4(input: String) { 
    // let probe = Command::new("sh")
    //     .arg("-c")
    //     .arg(format!("ffprobe -v error -select_streams v:0 -show_entries stream=codec_name \
    //     -of default=noprint_wrappers=1:nokey=1 {}", input))
    //     .output()
    //     .unwrap();


    // TODO: Windows compatability
    let mut ffmpeg = Command::new("sh")
        .arg("-c")
        .arg(format!("ffmpeg -y -i {} -c copy host_media.mp4", input))
        .spawn()
        .unwrap();

    loop {
        match &ffmpeg.try_wait() {
            Ok(_) => {
                if let Some(out) = ffmpeg.stdout{
                    println!("{:?}", out);                    
                }
                break;
            },
            Err(_) => {},
        };
    };
}


fn get_local_ip() -> Result<String, std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip().to_string())
}

#![allow(dead_code)]

use tokio::sync::oneshot;
use std::net::UdpSocket;

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

fn get_local_ip() -> Result<String, std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip().to_string())
}

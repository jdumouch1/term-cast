#![allow(dead_code)]

mod media;
mod app;

use std::{io::{ Write, stdout }, panic, panic::PanicInfo, sync::mpsc, sync::mpsc::Sender, thread, thread::JoinHandle,time::{ Duration, SystemTime }};
use app::{Controller, UIEvent};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers}, execute, terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, 
        enable_raw_mode, disable_raw_mode
    }};

type Result<T> = std::result::Result<T, UIError>;
type CrossTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug)]
enum UIError {}

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|info|{
        panic_hook(info);
    }));

    let ui_handle = start_ui().unwrap();
    ui_handle.join().unwrap();
}      

/// Swap the terminal to a TUI mode and begin a blocking UI loop. 
fn start_ui() -> Result<JoinHandle<()>> {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    
    // Spawn an event thread
    let (event_tx, event_rx) = mpsc::channel::<UIEvent>();
    spawn_event_loop(event_tx, 250).unwrap();


    // Main UI loop
    let handle = thread::spawn(move ||{
        // Create a model-controller with a shutdown sender
        let (mut controller, exit_rx) = Controller::new();

        loop {
            // Render TUI
            terminal.draw(|f|app::view::render(f, &controller.model)).unwrap();
            
            if let Ok(event) = event_rx.recv() {
                match event {
                    UIEvent::Input(key_ev) => {
                        // Handle <Ctrl+C>
                        if let KeyModifiers::CONTROL = key_ev.modifiers {
                            if key_ev.code == KeyCode::Char('c') { break; }
                        }
                        // Forward key presses to the model
                        controller.handle_key_event(key_ev);
                    }
                    UIEvent::Tick => {}
                }
            } 

            // Exit on a shutdown signal or if the shutdown sender is dropped
            match exit_rx.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => { 
                    break; 
                },
                _=> {}
            }
        }

        kill_terminal();
        ()
    });
        
    Ok(handle)
} 

/// Spawn a thread that hooks into user events as well as emits
/// a tick event at a given interval.
fn spawn_event_loop(event_tx: Sender<UIEvent>, tick_rate: u64) 
    -> Result<JoinHandle<()>> {
    
        let handle = thread::spawn(move || {
        let mut last_tick = SystemTime::now();
        let tick_rate = Duration::from_millis(tick_rate);
        loop {
            let elapsed = last_tick.elapsed().unwrap();
            if event::poll(tick_rate).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if let Err(_) = event_tx.send(UIEvent::Input(key)) {
                        break;
                    }
                }
            }

            if elapsed >= tick_rate {
                if let Err(_) = event_tx.send(UIEvent::Tick) { break; }
                last_tick = SystemTime::now();
            }
        }
    });

    Ok(handle)
}

#[allow(dead_code)]
/// Resize the terminal to force a complete redraw.
fn force_refresh(terminal: &mut CrossTerminal) -> Result<()> {
    let size = terminal.get_frame().size();
    terminal.resize(size).unwrap();
    Ok(())
}

/// Revert the terminal session to a normal state.
fn kill_terminal(){
    execute!(stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
}

/// Provides the the program a chance to revert the terminal 
/// to a normal state.  
fn panic_hook(info: &PanicInfo<'_>){
    kill_terminal();
    eprintln!("{:?}", info);
}
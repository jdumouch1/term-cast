#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cast;
mod media;
mod ui;

use event::KeyModifiers;
use ui::Model;
use std::{
    panic, 
    thread, 
    io::{ Write, stdout }, 
    panic::PanicInfo, 
    sync::mpsc, 
    thread::JoinHandle, 
    sync::mpsc::Sender, 
    time::{ Duration, SystemTime }
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, 
        enable_raw_mode, disable_raw_mode
    },
    event::{
        self, DisableMouseCapture, EnableMouseCapture, 
        Event, KeyCode, KeyEvent,
    },
    execute,
};

type Result<T> = std::result::Result<T, UIError>;
type CrossTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug)]
enum UIError {}

enum UIEvent {
    Input(KeyEvent),
    Tick,
}

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|info|{
        panic_hook(info);
    }));

    start_ui().unwrap();
}      

/// Swap the terminal to a TUI mode and begin a blocking UI loop. 
fn start_ui() -> Result<()> {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // Create a model-controller with a shutdown sender
    let (exit_tx, exit_rx) = mpsc::channel::<()>();
    let mut model = Model::new(exit_tx);
    
    // Spawn an event thread
    let (event_tx, event_rx) = mpsc::channel::<UIEvent>();
    spawn_event_loop(event_tx, 250).unwrap();

    // Main UI loop
    loop {
        terminal.draw(|f| ui::view::render(f, &model)).unwrap();

        if let Ok(event) = event_rx.recv() {
            match event {
                UIEvent::Input(key_ev) => {
                    if let KeyModifiers::CONTROL = key_ev.modifiers {
                        if key_ev.code == KeyCode::Char('c'){
                            break;
                        }
                    }

                    // Forward key presses to the model
                    model.handle_key_event(key_ev);
            }
                UIEvent::Tick => {}
            }
        } 

        // Exit on a shutdown signal or if the shutdown sender is dropped
        match exit_rx.try_recv() {
            Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {break;},
            _=> {}
        }
    }

    kill_terminal();
        
    Ok(())
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
                    event_tx.send(UIEvent::Input(key)).unwrap();
                }
            }

            if elapsed >= tick_rate {
                event_tx.send(UIEvent::Tick).unwrap();
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
use std::{collections::VecDeque, path::PathBuf, sync::mpsc::{self, Receiver, Sender}};
use crossterm::{event::{KeyEvent}};
use tui::{style::{Modifier, Style}, text::{Span, Spans}};


pub mod view;
mod tests;
mod input;

#[derive(Eq, PartialEq)]
pub enum LogLevel {
    General,
    Debug,
    Error,  
}

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Control,
    Input,
    Help,
}

pub enum UIEvent {
    Input(KeyEvent),
    Tick,
}

pub struct Controller {
    pub model: Model,
    shutdown_tx: Sender<()>,
}

impl Controller {
    pub fn new() -> (Self, Receiver<()>) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let controller = Self {
            model: Model::default(),
            shutdown_tx,
        };

        ( controller, shutdown_rx )
    }

    /// Push a new log entry into the model.   
    /// All log entries share the same buffer, exceeding this capacity
    /// with drop the oldest log entry.
    pub fn log(&mut self, level: LogLevel, entry: String) {
        let log = &mut self.model.log; 
        // Pop oldest item if over capacity
        if log.len() == log.capacity() { log.pop_back(); }
        log.push_front((level, entry));
    }

    /// Uses a passed KeyEvent to update the model appropriately.  
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        // Pass event to appropriate handler
        match self.model.mode {
            Mode::Input => self.input_mode_handler(event),
            Mode::Help => self.help_mode_handler(event),
            Mode::Control => self.control_mode_handler(event),
        }
    }

    /// Attempt to queue casting the filepath stored 
    /// in the model's input_string.  
    fn queue_file(&mut self) -> Option<String> {
        let in_str = String::from(&self.model.input_string);
        
        // Expand tilde to home
        let mut path: PathBuf;
        if in_str.len() > 3 && &in_str[..1] == "~" {
            path = dirs::home_dir().unwrap();
            path.push(&in_str[2..]);
        }else{
            path = PathBuf::from(&in_str);
        }

        // Ensure the file exists
        if !path.is_file() {
            self.log(
                LogLevel::Error, 
                format!("\"{}\" is not a valid file.", in_str));
            self.model.selected_file = None;
            return None
        } 
        // Convert to absolute path
        path = std::fs::canonicalize(&path).unwrap();

        // Update model and return the path
        let path_str = String::from(path.to_str().unwrap());
        self.model.selected_file = Some(path); 
        self.log(
            LogLevel::General,
            format!("\"{}\" selected as active file.", &path_str));
            
        Some(path_str)
    }
}

pub struct Model {
    pub mode: Mode,                 // Switch input event handling
    pub input_cursor: usize,        // Cursor for text editing/input  
    pub input_string: String,       // String buffer for input text
    pub log: VecDeque<(LogLevel, String)>,    // Buffer for logs
    pub selected_file: Option<PathBuf>,
}

impl Model {
    fn default() -> Self {
        Self {
            mode: Mode::Control,
            input_cursor: 0,
            input_string: String::default(),
            log: VecDeque::with_capacity(1024),
            selected_file: None,
        }
    }

    /// Returns a Spans representing the input text.   
    /// A carat will be added to represent the cursor position
    /// if the model is in input_mode, otherwise the raw text will be
    /// returned.
    pub fn get_input_span(&self) -> Spans {
        let in_str = &self.input_string;
        let cursor = self.input_cursor;
        match self.mode {
            Mode::Input => {
                // Build a Spans with emphasized cursor
                let mut spans: Vec<Span> = Vec::default();
                if in_str.len() > 0 {
                    if cursor > 0 {
                    // Text before cursor
                    spans.push(Span::from(&in_str[..cursor])); 
                    }
                    if cursor <= in_str.len()-1 {
                        // Cursor on text
                        spans.push(Span::styled(&in_str[cursor..cursor+1],
                            Style::default().add_modifier(Modifier::ITALIC)));
                        // Text after cursor
                        if !&in_str[cursor+1..].is_empty() {
                            spans.push(Span::from(&in_str[cursor+1..]));
                        }
                    }
                }
                if cursor == in_str.len() {
                    // Cursor past string
                    spans.push(Span::styled("_",
                        Style::default().add_modifier(Modifier::ITALIC)));
                }
                
                Spans::from(spans)
            },
            _ => Spans::from(&in_str[..]),
        }
    }
}


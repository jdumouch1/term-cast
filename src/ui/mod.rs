use std::{sync::mpsc::Sender, collections::VecDeque};
use crossterm::{event::{KeyCode, KeyEvent, KeyModifiers}, style::style};
use tui::{widgets::ListItem, style::{Modifier, Style}, text::{Span, Spans}};

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

pub struct Model {
    mode: Mode,                 // Switch input event handling
    input_cursor: usize,        // Cursor for text editing/input  
    input_string: String,       // String buffer for input text
    log_items: VecDeque<(LogLevel, String)>,    // Buffer for logs
    shutdown_tx: Sender<()>,    // Sender to close program
}

impl Model {
    pub fn new(shutdown_tx: Sender<()>) -> Self {
        Self {
            mode: Mode::Control,
            input_cursor: 0,
            input_string: String::default(),
            log_items: VecDeque::with_capacity(1024),
            shutdown_tx,
        }
    }

    /// Uses a passed KeyEvent to update the model appropriately.  
    /// This internally handles input modes.
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        // Pass event to appropriate handler
        match self.mode {
            Mode::Input => self.input_mode_handler(event),
            Mode::Help => self.help_mode_handler(event),
            Mode::Control => self.control_mode_handler(event),
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
                        spans.push(Span::from(&in_str[cursor+1..]));

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
    
    /// Push a new log entry into the model.   
    /// All log entries share the same buffer, exceeding this capacity
    /// with drop the oldest log entry.
    pub fn log(&mut self, level: LogLevel, entry: String) {
        if self.log_items.len() == self.log_items.capacity() {
            self.log_items.pop_back();
        }

        self.log_items.push_front((level, entry));
    }

    /// Return a reference to the log deque.
    pub fn get_log_items(&self) -> &VecDeque<(LogLevel, String)> {
        &self.log_items
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }
}


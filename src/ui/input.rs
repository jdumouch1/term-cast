use crossterm::event::{KeyEvent, KeyModifiers};
use warp::body::form;

use super::LogLevel;

impl super::Model {
    /// Provides key handling for operation of the program.
    pub fn control_mode_handler(&mut self, event: KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Enter => {}
            crossterm::event::KeyCode::Left => {}
            crossterm::event::KeyCode::Right => {}
            crossterm::event::KeyCode::Up => {}
            crossterm::event::KeyCode::Down => {}
            crossterm::event::KeyCode::Delete => {}
            crossterm::event::KeyCode::Char(ch) => {
                match ch {
                    '/' => { self.input_mode = true; }
                    'q' => { self.shutdown_tx.send(()).unwrap();},
                    _=>{}
                }
                
                
            }
            _ => {}
        }
    }
    /// Handle key events for situations in which the user 
    /// is entering a string.   
    /// This provides basic text input/editing functionality.
    /// *Note: This does not correctly handle unicode.*
    pub fn input_mode_handler(&mut self, event: KeyEvent) {
        self.log(LogLevel::General, format!("{:?}", event));
        
        let len = &self.input_string.len();
        let cursor = &mut self.input_cursor;
        let in_str = &mut self.input_string;
        match event.code {
            crossterm::event::KeyCode::Esc => {self.input_mode = false;}
            crossterm::event::KeyCode::Delete => {
                // Delete the char on the cursor
                if *len > 0 && *cursor < *len {
                    // Pop if char is last in string
                    if *cursor < *len - 1 { in_str.remove(*cursor); }
                    else { in_str.pop(); }
                }
            }
            crossterm::event::KeyCode::Backspace => {
                // Delete the char behind the cursor
                if *len > 0 && *cursor > 0 {
                    // Pop if char is last in string
                    if *cursor < *len { in_str.remove(*cursor-1); }
                    else { in_str.pop(); }
                    *cursor -= 1; // Decrement the cursor regardless
                }                
            }
            crossterm::event::KeyCode::Enter => {
                // Run active element
                self.input_mode = false;
            }
            crossterm::event::KeyCode::Left => {
                if *cursor > 0 {*cursor -= 1;}  
            }
            crossterm::event::KeyCode::Right => {
                if *cursor < *len {*cursor += 1;}
            }
            crossterm::event::KeyCode::Home => {*cursor = 0;}
            crossterm::event::KeyCode::End => {*cursor = *len;}
            crossterm::event::KeyCode::Char(ch) => {
                in_str.insert(*cursor, ch);
                *cursor+=1;
            }
            _ => {}
        }
    }


}
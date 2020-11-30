use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use warp::body::form;

use super::Mode;


impl super::Model {

    /// Provides key handling while in the help menu
    pub fn help_mode_handler(&mut self, event: KeyEvent){
        match event.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = Mode::Control;
            }
            KeyCode::Char(ch) => {
                match ch {
                    'q' => self.mode = Mode::Control, 
                    _=>{}
                }
            }
            _=>{}
        }
    }

    /// Provides key handling for operation of the program.
    pub fn control_mode_handler(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Enter => {}
            KeyCode::Left => {}
            KeyCode::Right => {}
            KeyCode::Up => {}
            KeyCode::Down => {}
            KeyCode::Delete => {}
            KeyCode::Char(ch) => {
                match ch {
                    '?' => { self.mode = Mode::Help; }
                    '/' => { self.mode = Mode::Input; }
                    'q' => { self.shutdown_tx.send(()).unwrap(); },
                    _ => {},
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
        let len = &self.input_string.len();
        let cursor = &mut self.input_cursor;
        let in_str = &mut self.input_string;
        match event.code {
            KeyCode::Esc => {self.mode = Mode::Control;}
            KeyCode::Delete => {
                // Delete the char on the cursor
                if *len > 0 && *cursor < *len {
                    // Pop if char is last in string
                    if *cursor < *len - 1 { in_str.remove(*cursor); }
                    else { in_str.pop(); }
                }
            }
            KeyCode::Backspace => {
                // Delete the char behind the cursor
                if *len > 0 && *cursor > 0 {
                    // Pop if char is last in string
                    if *cursor < *len { in_str.remove(*cursor-1); }
                    else { in_str.pop(); }
                    *cursor -= 1; // Decrement the cursor regardless
                }                
            }
            KeyCode::Enter => {
                // Run active element
                self.mode = super::Mode::Control;
            }
            KeyCode::Left => {
                if *cursor > 0 {*cursor -= 1;}  
            }
            KeyCode::Right => {
                if *cursor < *len {*cursor += 1;}
            }
            KeyCode::Home => {*cursor = 0;}
            KeyCode::End => {*cursor = *len;}
            KeyCode::Char(ch) => {
                in_str.insert(*cursor, ch);
                *cursor+=1;
            }
            _ => {}
        }
    }


}
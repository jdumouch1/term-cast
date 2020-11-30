use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent};
use rust_cast::channels::media::Status;

enum CursorDir {
    Left,
    Right,
}
pub enum State {
    Main,
    Search,
}

struct ModelState<S> {
    _inner: S,
}

pub struct Model {
    shutdown_tx: Sender<()>,
    pub state: State,
    pub search_cursor: usize,
    pub filepath: String,
}

impl Model {
    pub fn new(shutdown_tx: Sender<()>) -> Self {
        Self {
            shutdown_tx,
            state: State::Main,
            search_cursor: 0, 
            filepath: String::with_capacity(128)
        }
    }
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match self.state {
            State::Main => {
                match event.code {
                    KeyCode::Esc => { self.shutdown_tx.send(()).unwrap(); }
                    KeyCode::Enter => {}
                    KeyCode::Left => {}
                    KeyCode::Right => {}
                    KeyCode::Up => {}
                    KeyCode::Down => {}
                    KeyCode::Home => {}
                    KeyCode::End => {}
                    KeyCode::Delete => {}
                    KeyCode::Char(ch) => {
                        match ch {
                            '/' => { self.state = State::Search }
                            _ => {}
                        }
                    }
                    _=>{}
                }
            }
            State::Search => {self.handle_search_input(event)}
        }
    }

    fn handle_search_input(&mut self, event: KeyEvent){
        let path_len = self.filepath.len();
        match event.code {
            KeyCode::Esc => {self.shutdown_tx.send(()).unwrap();}
            KeyCode::Insert => {}
            KeyCode::Enter => { self.state = State::Main; }
            KeyCode::Backspace => {
                if path_len > 0 || self.search_cursor > 0 {
                    if self.search_cursor != path_len {
                        self.filepath.remove(self.search_cursor);
                    }else{
                        self.filepath.pop();
                        self.move_search_cursor(CursorDir::Left);
                    }
                }
            }
            KeyCode::Left => { self.move_search_cursor(CursorDir::Left) }
            KeyCode::Right => { self.move_search_cursor(CursorDir::Right) }
            KeyCode::Home => { self.search_cursor = 0; }
            KeyCode::End => { self.search_cursor = path_len; }
            KeyCode::Delete => {
                if self.filepath.len() > 0 && 
                    self.search_cursor < path_len {
                    if self.search_cursor < path_len-1{
                        self.filepath.remove(self.search_cursor);
                    }else{
                        self.filepath.pop();
                    } 
                }
            }
            KeyCode::Char(ch) => {
                self.filepath.insert(self.search_cursor, ch);
                self.search_cursor += 1;
            }
            _ => {}
        }
    }

    fn move_search_cursor(&mut self, dir: CursorDir){
        match dir {
            CursorDir::Left => {
                if self.search_cursor > 0 {
                    self.search_cursor -= 1;
                }
            }
            CursorDir::Right => {
                if self.search_cursor < self.filepath.len() {
                    self.search_cursor += 1;
                }
            }
        }
    }
}


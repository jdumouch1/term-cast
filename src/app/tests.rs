#![allow(unused_imports)]
mod controller {
    use std::{env::{consts::OS, current_exe}, fs::File, path::PathBuf, fs::remove_file};

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::app::{Controller, Mode};

    #[test]
    fn input_mode_backspace(){   
        let backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        // Setup the model in input mode
        let (mut c, _) = Controller::new();
        c.model.mode = Mode::Input;

        // 0 Length string
        c.handle_key_event(backspace.clone());
        assert_eq!(c.model.input_string, String::default());

        c.model.input_string = String::from("foo bar");
        
        // Test at index 0
        c.handle_key_event(backspace.clone());
        assert_eq!(c.model.input_string, String::from("foo bar"));

        // Delete from the middle of the string
        c.model.input_cursor = 4;
        c.handle_key_event(backspace.clone());
        assert_eq!(c.model.input_string, String::from("foobar"));

        // Test at end of string
        c.model.input_cursor = c.model.input_string.len();
        c.handle_key_event(backspace);
        assert_eq!(c.model.input_string, String::from("fooba"));
    }

    #[test]
    fn input_mode_delete(){ 
        let delete = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);

        // Setup the model in input mode
        let (mut c, _) = Controller::new();
        c.model.mode = Mode::Input;

        // 0 Length string
        c.handle_key_event(delete.clone());
        assert_eq!(c.model.input_string, String::default());

        c.model.input_string = String::from("foo bar");
        
        // Test at index 0
        c.handle_key_event(delete.clone());
        assert_eq!(c.model.input_string, String::from("oo bar"));

        // Delete from the middle of the string
        c.model.input_cursor = 2;
        c.handle_key_event(delete.clone());
        assert_eq!(c.model.input_string, String::from("oobar"));

        // Test at end of string
        c.model.input_cursor = c.model.input_string.len();
        c.handle_key_event(delete);
        assert_eq!(c.model.input_string, String::from("oobar"));
    }

    #[test]
    fn input_mode_esc() {
        // Setup the model in input mode
        let (mut c, _) = Controller::new();
        c.model.mode = Mode::Input;

        // Test that esc cancelled input mode
        c.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_ne!(c.model.mode, Mode::Input);
    }

    #[test]
    /// Test that queue files correctly reads directories   
    /// *Note: these tests will fail if requisite write permissions are 
    /// not met.*
    fn queue_file() {   
        // Create a test file
        let mut path = dirs::home_dir().unwrap();
        path.push("term-cast-test");
        File::create(&path).unwrap();

        // Queue up the test file using the absolute path
        let (mut c, _) = Controller::new();
        c.model.input_string = String::from(path.to_str().unwrap());
        assert!(c.queue_file().is_some());

        // Queue the test file using tilde with absolute path
        if OS == "linux" || OS == "macos" {
            c.model.input_string = String::from("~/term-cast-test");
        } else if OS == "windows" {
            c.model.input_string = String::from(r"~\term-cast-test");
        }
        else {panic!("Unsupported OS");}

        assert!(c.queue_file().is_some());

        // Remove the test file
        std::fs::remove_file(&path).unwrap();

        // Create a test file using a relative path
        path = PathBuf::from("term-cast-test");
        File::create(&path).unwrap();
        c.model.input_string = String::from("./term-cast-test");
        assert!(c.queue_file().is_some());
        c.model.input_string = String::from("term-cast-test");
        assert!(c.queue_file().is_some());
        // Remove the test file
        std::fs::remove_file(&path).unwrap();

        // Create a test file using relative previous dir
        path = PathBuf::from("../term-cast-test");
        File::create(&path).unwrap();
        c.model.input_string = String::from("../term-cast-test");
        assert!(c.queue_file().is_some());
        std::fs::remove_file(&path).unwrap();
    }

}

mod model {
    use tui::{style::{Modifier, Style}, text::{Span, Spans}};
    use crate::app::{Controller, Mode};

    #[test]
    fn get_input_span() {
        let (mut c, _) = Controller::new();
        let mut model = &mut c.model; 
        model.mode = Mode::Input;
        model.input_string = String::from("cursor");
    
        // Cursor = 0, highlighted c
        let mut spans = Spans::from(vec![
            Span::styled("c", Style::default().add_modifier(Modifier::ITALIC)),
            Span::from("ursor"),
        ]); 
        assert_eq!(model.get_input_span(), spans);
    
        // Cursor = 3, highlighted s
        spans = Spans::from(vec![
            Span::from("cur"),
            Span::styled("s", Style::default().add_modifier(Modifier::ITALIC)),
            Span::from("or"),
        ]); 
        model.input_cursor = 3;
        assert_eq!(model.get_input_span(), spans);
    
        // Cursor = 5, highlighted r
        spans = Spans::from(vec![
            Span::from("curso"),
            Span::styled("r", Style::default().add_modifier(Modifier::ITALIC)),
        ]); 
        model.input_cursor = 5;
        assert_eq!(model.get_input_span(), spans);
    
        // Cursor = 6, highlighted _ after cursor
        spans = Spans::from(vec![
            Span::from("cursor"),
            Span::styled("_", Style::default().add_modifier(Modifier::ITALIC)),
        ]); 
        model.input_cursor = 6;
        assert_eq!(model.get_input_span(), spans);
    }

}



use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn input_mode_backspace(){   
    let backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

    // Setup the model in input mode
    let (tx, _rx) = std::sync::mpsc::channel::<()>();
    let mut model = super::Model::new(tx);
    model.mode = super::Mode::Input;

    // 0 Length string
    model.handle_key_event(backspace.clone());
    assert_eq!(model.input_string, String::default());

    model.input_string = String::from("foo bar");
    
    // Test at index 0
    model.handle_key_event(backspace.clone());
    assert_eq!(model.input_string, String::from("foo bar"));

    // Delete from the middle of the string
    model.input_cursor = 4;
    model.handle_key_event(backspace.clone());
    assert_eq!(model.input_string, String::from("foobar"));

    // Test at end of string
    model.input_cursor = model.input_string.len();
    model.handle_key_event(backspace);
    assert_eq!(model.input_string, String::from("fooba"));
}

#[test]
fn input_mode_delete(){ 
    let delete = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);

    // Setup the model in input mode
    let (tx, _rx) = std::sync::mpsc::channel::<()>();
    let mut model = super::Model::new(tx);
    model.mode = super::Mode::Input;


    // 0 Length string
    model.handle_key_event(delete.clone());
    assert_eq!(model.input_string, String::default());

    model.input_string = String::from("foo bar");
    
    // Test at index 0
    model.handle_key_event(delete.clone());
    assert_eq!(model.input_string, String::from("oo bar"));

    // Delete from the middle of the string
    model.input_cursor = 2;
    model.handle_key_event(delete.clone());
    assert_eq!(model.input_string, String::from("oobar"));

    // Test at end of string
    model.input_cursor = model.input_string.len();
    model.handle_key_event(delete);
    assert_eq!(model.input_string, String::from("oobar"));
}

#[test]
fn input_mode_esc() {
    // Setup the model in input mode
    let (tx, _rx) = std::sync::mpsc::channel::<()>();
    let mut model = super::Model::new(tx);
    model.mode = super::Mode::Input;


    // Test that esc cancelled input mode
    model.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_ne!(model.mode, super::Mode::Input);
}
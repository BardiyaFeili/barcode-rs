use crossterm::event::{self, Event, KeyEvent};
use std::{io, time::Duration};

#[derive(Debug)]
pub enum InputEvent {
    Key(KeyEvent),
    None,
}

pub fn read_input() -> io::Result<InputEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            // capture all keys and modifiers as-is
            Ok(InputEvent::Key(key_event))
        } else {
            Ok(InputEvent::None)
        }
    } else {
        Ok(InputEvent::None)
    }
}

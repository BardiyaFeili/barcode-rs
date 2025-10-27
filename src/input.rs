use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::io;
use std::time::Duration;

pub enum InputEvent {
    Key(KeyEvent),
    Quit,
    None,
}

pub fn read_input() -> io::Result<InputEvent> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => {
                Ok(InputEvent::Key(key))
            }
            _ => Ok(InputEvent::None),
        }
    } else {
        Ok(InputEvent::None)
    }
}

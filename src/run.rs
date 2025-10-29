use std::error::Error;

use crate::{
    action::{Action, take_action},
    args::parse_args,
    component::{Component, ComponentType},
    file::open_file,
    input::read_input,
    log::{log, log_startup},
    modal::{Mode, handle_mode_input},
    render,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut mode = Mode::Normal;
    let mut active_components: Vec<Component> = Vec::new();

    log_startup("Barcode", "pre-alpha")?;
    let last = startup(&mut active_components)?;

    loop {
        for component in active_components.iter_mut() {
            Component::update(component)?
        }

        let action = handle_mode_input(&mut mode, read_input()?);


        match action {
            Action::Quit => break,
            _ => take_action(&action, last, &mut active_components)?,
        }

        render::render(&mut active_components)?;
    }

    Ok(())
}

fn startup(active_components: &mut Vec<Component>) -> Result<usize, Box<dyn Error>> {
    let args = parse_args();

    for file in &args.files {
        let content = open_file(file)?;
        let content = content.lines().map(|s| s.to_string()).collect();
        active_components.push(Component::new(content, ComponentType::Buffer));
    }

    Ok(args.files.len())
}

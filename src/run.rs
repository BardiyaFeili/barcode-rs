use std::error::Error;

use crate::{
    action::{Action, take_action}, args::{Args, parse_args}, component::{Component, ComponentType}, config::resolve_config_files, file::open_file, input::read_input, log::log_startup, modal::{Mode, handle_mode_input}, render::render
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut mode = Mode::Normal;
    let mut active_components: Vec<Component> = Vec::new();
    let args = parse_args();

    let last = startup(&args, &mut active_components)?;
    if args.only_startup{
        return Ok(());
    }

    loop {
        for component in active_components.iter_mut() {
            Component::update(component)?
        }

        let action = handle_mode_input(&mut mode, read_input()?);

        match action {
            Action::Quit => break,
            _ => take_action(&action, last, &mut active_components)?,
        }

        render(&mut active_components)?;
    }

    Ok(())
}

fn startup(args: &Args, active_components: &mut Vec<Component>) -> Result<usize, Box<dyn Error>> {
    log_startup("Barcode", "pre-alpha")?;

    for file in &args.files {
        let content = open_file(file)?;
        let content = content.lines().map(|s| s.to_string()).collect();
        active_components.push(Component::new(content, ComponentType::Buffer));
    }
    
    resolve_config_files(&args)?;

    Ok(args.files.len())
}

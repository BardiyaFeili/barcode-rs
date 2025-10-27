use std::error::Error;

use crate::{
    args::parse_args,
    component::{Action, Component, ComponentType, handle_write_action},
    file::open_file,
    input::read_input,
    modal::{Mode, handle_mode_input},
    render,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut mode = Mode::Normal;
    let mut active_components: Vec<Component> = Vec::new();
    let mut focused: Option<&mut Component> = None;

    let last = startup(&mut active_components, &mut focused)?;

    loop {
        for component in active_components.iter_mut() {
            Component::update(component)
        }
        let action = handle_mode_input(&mut mode, read_input()?);
        match action {
            Action::TextAction(a) => handle_write_action(active_components.get_mut(last - 1), a)?,
            _ => (),
        }
        if mode == Mode::Quit {
            break;
        }
        render::render(&mut active_components)?;
    }

    Ok(())
}

fn startup(
    active_components: &mut Vec<Component>,
    focused: &mut Option<&mut Component>,
) -> Result<usize, Box<dyn Error>> {
    let args = parse_args();

    for file in &args.files {
        let content = open_file(file)?;
        let content = content.lines().map(|s| s.to_string()).collect();
        active_components.push(Component::new(content, ComponentType::Buffer));
    };

    Ok(args.files.len())
}

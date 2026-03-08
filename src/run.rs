use std::error::Error;
use std::time::{Duration, Instant};

use crate::log::log;
use crate::{
    action::{Action, take_action},
    args::Args,
    component::{Component, ComponentType},
    config::resolve_config_files,
    file::open_file,
    input::read_input,
    log::log_startup,
    modal::{Mode, handle_mode_input},
    render::render,
    status_line::update_status_line,
};

pub struct Editor {
    pub mode: Mode,
    pub last_mode: Mode,
    pub config: crate::config::Config,
    pub active_components: Vec<Component>,
    pub focused_idx: usize,
    pub last_status_update: Instant,
}

impl Editor {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        let config = resolve_config_files(&args)?;
        let mut active_components = Vec::new();
        let last_count = startup(&args, &mut active_components)?;

        let mut focused_idx = last_count.saturating_sub(1);

        // Ensure initial focused index is a focusable component
        if !active_components.is_empty() {
            while !active_components[focused_idx].focusable && focused_idx > 0 {
                focused_idx -= 1;
            }
        }

        // Add status line if enabled
        if config.status_line.enabled {
            let mut status_line =
                Component::new(vec![String::new()], ComponentType::StatusLine, None);
            status_line.window.window_height = 1;
            active_components.push(status_line);
        }

        Ok(Self {
            mode: Mode::Normal,
            last_mode: Mode::Normal,
            config,
            active_components,
            focused_idx,
            last_status_update: Instant::now() - Duration::from_secs(60),
        })
    }

    pub fn update(&mut self, delta: Duration) -> Result<bool, Box<dyn Error>> {
        let mut should_redraw = false;

        // Only update status line if enabled and some time has passed, or if something changed.
        let needs_status_update = (self.mode != self.last_mode) || self.active_components.get(self.focused_idx)
            .map(|c| c.needs_update).unwrap_or(false);

        if self.config.status_line.enabled && (self.last_status_update.elapsed() > Duration::from_secs(1) || needs_status_update) {
            update_status_line(
                &mut self.active_components,
                &self.config.status_line,
                self.mode,
                self.focused_idx,
                terminal_width()?,
            )?;
            self.last_status_update = Instant::now();
            should_redraw = true;
        }
        self.last_mode = self.mode;

        // Update all components and remove expired ones
        let mut i = 0;
        while i < self.active_components.len() {
            self.active_components[i].update(delta)?;
            if self.active_components[i].is_expired() {
                self.active_components.remove(i);
                should_redraw = true;
                // Adjust focus if needed
                if self.focused_idx >= self.active_components.len()
                    && !self.active_components.is_empty()
                {
                    self.focused_idx = self.active_components.len() - 1;
                }
            } else {
                if self.active_components[i].needs_update {
                    should_redraw = true;
                }
                i += 1;
            }
        }
        Ok(should_redraw)
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        render(
            &mut self.active_components,
            &self.mode,
            self.focused_idx,
            &self.config,
        )
    }
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut editor = Editor::new(args)?;

    if editor.active_components.is_empty() {
        return Ok(());
    }

    let mut last_tick = Instant::now();
    let mut needs_redraw = true;

    loop {
        let delta = last_tick.elapsed();
        last_tick = Instant::now();

        if editor.update(delta)? {
            needs_redraw = true;
        }

        if needs_redraw {
            editor.draw()?;
            needs_redraw = false;
        }

        // Wait for input with a timeout to avoid busy-waiting
        let timeout = editor.active_components.iter()
            .filter_map(|c| c.timer)
            .min()
            .unwrap_or(Duration::from_millis(100));

        if crossterm::event::poll(timeout)? {
            let old_mode = editor.mode;
            let action = handle_mode_input(&mut editor.mode, read_input()?);

            if editor.mode != old_mode {
                log(format!("Mode change: {:?} -> {:?}", old_mode, editor.mode))?;
            }

            match action {
                Action::Quit => break,
                _ => take_action(
                    &action,
                    &mut editor.focused_idx,
                    &mut editor.active_components,
                    &editor.mode,
                    old_mode,
                    &editor.config,
                )?,
            }

            if editor.active_components.is_empty() {
                break;
            }
            needs_redraw = true;
        }
    }

    Ok(())
}

fn terminal_width() -> Result<u16, Box<dyn Error>> {
    let (w, _) = crossterm::terminal::size()?;
    Ok(w)
}

fn startup(args: &Args, active_components: &mut Vec<Component>) -> Result<usize, Box<dyn Error>> {
    log_startup("Barcode", "pre-alpha")?;

    for file in &args.files {
        let content_str = open_file(file)?;
        let mut content: Vec<String> = content_str.lines().map(|s| s.to_string()).collect();
        if content.is_empty() {
            content.push("".to_string());
        }
        active_components.push(Component::new(
            content,
            ComponentType::Buffer,
            Some(file.clone()),
        ));
    }

    Ok(active_components.len())
}

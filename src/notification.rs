use crate::{
    component::{Component, ComponentType},
    config::Config,
};
use std::error::Error;
use std::time::Duration;

pub fn push_notification(
    components: &mut Vec<Component>,
    message: String,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let cfg = &config.notification;
    if !cfg.enabled {
        return Ok(());
    }

    let mut notify = Component::new(vec![message], ComponentType::Notification, None, config)
        .with_timer(Duration::from_secs(cfg.timeout_secs));

    notify.window.window_type = crate::window::WindowType::Floating;
    notify.window.h_anchor = cfg.h_anchor;
    notify.window.v_anchor = cfg.v_anchor;
    notify.window.x = 2;
    notify.window.y = 1;
    notify.window.window_width = 30;
    notify.window.window_height = 3;
    notify.window.border_style = cfg.border_style;

    components.push(notify);
    Ok(())
}

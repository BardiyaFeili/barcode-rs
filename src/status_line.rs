use crate::{
    component::{Component, ComponentType},
    config::StatusLineConfig,
    modal::Mode,
};
use std::error::Error;
use std::path::Path;

struct StatusLineInfo<'a> {
    file_name: &'a str,
    dir_name: &'a str,
    line_num: u16,
    col_num: u16,
    percent: &'a str,
}

pub fn update_status_line(
    components: &mut [Component],
    config: &StatusLineConfig,
    mode: Mode,
    focused_idx: usize,
    term_w: u16,
) -> Result<(), Box<dyn Error>> {
    let mut file_name = "[No Name]".to_string();
    let mut dir_name = ".".to_string();
    let mut line_num = 0;
    let mut col_num = 0;
    let mut percent = "Top".to_string();

    if focused_idx < components.len() {
        let active = &components[focused_idx];
        let path = active.file_path.as_ref();
        file_name = path
            .and_then(|p| Path::new(p).file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("[No Name]")
            .to_string();

        dir_name = match path {
            Some(p) => {
                let p = Path::new(p);
                let abs_path = if p.is_absolute() {
                    p.parent().unwrap_or(p).to_path_buf()
                } else {
                    let mut current =
                        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    if let Some(parent) = p.parent() {
                        current.push(parent);
                    }
                    current
                };

                let abs_str = abs_path.to_str().unwrap_or(".");
                if let Ok(home) = std::env::var("HOME") {
                    if abs_str.starts_with(&home) {
                        abs_str.replace(&home, "~")
                    } else {
                        abs_str.to_string()
                    }
                } else {
                    abs_str.to_string()
                }
            }
            None => ".".to_string(),
        };

        line_num = active.cursor.y + 1;
        col_num = active.cursor.x + 1;

        let total_lines = active.content.len();
        if total_lines > 0 {
            let p = (line_num as f32 / total_lines as f32 * 100.0) as u32;
            percent = if line_num == 1 {
                "Top".to_string()
            } else if line_num == total_lines as u16 {
                "Bot".to_string()
            } else {
                format!("{}%", p)
            };
        }
    }

    let info = StatusLineInfo {
        file_name: &file_name,
        dir_name: &dir_name,
        line_num,
        col_num,
        percent: &percent,
    };

    let formatted = format_status_line(config, mode, info, term_w);

    if let Some(status) = components
        .iter_mut()
        .find(|c| c.component_type == ComponentType::StatusLine)
    {
        status.content[0] = formatted;
    }

    Ok(())
}

fn format_status_line(
    config: &StatusLineConfig,
    mode: Mode,
    info: StatusLineInfo,
    term_w: u16,
) -> String {
    use chrono::Local;
    use std::sync::OnceLock;
    use unicode_width::UnicodeWidthStr;

    static USER_CACHE: OnceLock<String> = OnceLock::new();
    static HOST_CACHE: OnceLock<String> = OnceLock::new();

    let mode_str = format!("{:?}", mode).to_uppercase();
    let time_str = Local::now().format("%H:%M").to_string();
    let date_str = Local::now().format("%Y-%m-%d").to_string();

    let user_str = USER_CACHE.get_or_init(|| {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string())
    });

    let host_str = HOST_CACHE.get_or_init(|| {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| {
                std::fs::read_to_string("/proc/sys/kernel/hostname")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "host".to_string())
            })
    });

    let line_str = info.line_num.to_string();
    let col_str = info.col_num.to_string();
    let cursor_str = format!("{}:{}", line_str, col_str);

    let replace_vars = |text: &str| -> String {
        text.replace("{mode}", &mode_str)
            .replace("{file}", info.file_name)
            .replace("{dir}", info.dir_name)
            .replace("{time}", &time_str)
            .replace("{date}", &date_str)
            .replace("{user}", user_str)
            .replace("{host}", host_str)
            .replace("{line}", &line_str)
            .replace("{col}", &col_str)
            .replace("{cursor}", &cursor_str)
            .replace("{percent}", info.percent)
    };

    let left = replace_vars(&config.text_left);
    let center = replace_vars(&config.text_center);
    let right = replace_vars(&config.text_right);

    let left_end_w = config.left_end.width();
    let right_end_w = config.right_end.width();

    let available_w = (term_w as usize).saturating_sub(left_end_w + right_end_w);

    let left_w = left.width();
    let center_w = center.width();
    let right_w = right.width();

    let total_text_w = left_w + center_w + right_w;

    if total_text_w >= available_w {
        let combined = format!("{} {} {}", left, center, right);
        if combined.width() > available_w {
            // Truncate to fit
            let mut res = String::new();
            let mut curr_w = 0;
            for c in combined.chars() {
                let cw = UnicodeWidthStr::width(c.to_string().as_str());
                if curr_w + cw > available_w {
                    break;
                }
                res.push(c);
                curr_w += cw;
            }
            // Pad if still short due to multi-byte chars
            if curr_w < available_w {
                res.push_str(&" ".repeat(available_w - curr_w));
            }
            return res;
        } else {
            return format!("{}{}", combined, " ".repeat(available_w - combined.width()));
        }
    }

    let left_part_end = left_w;
    let right_part_start = available_w - right_w;

    let center_len = center_w;
    let center_start = if center_len > 0 {
        (available_w / 2).saturating_sub(center_len / 2)
    } else {
        left_part_end
    };

    let center_start = center_start.max(left_part_end);
    let center_end = (center_start + center_len).min(right_part_start);

    let space1_len = center_start - left_part_end;
    let space2_len = right_part_start - center_end;

    format!(
        "{}{}{}{}{}",
        left,
        " ".repeat(space1_len),
        if center_end > center_start {
            &center
        } else {
            ""
        },
        " ".repeat(space2_len),
        right
    )
}

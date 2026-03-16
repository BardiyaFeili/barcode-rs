use crossterm::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(default)]
pub struct Theme {
    #[serde(deserialize_with = "de_color_opt", serialize_with = "ser_color_opt")]
    pub bg: Option<Color>,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub border: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub status_bg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub status_fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub accent: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub selection_bg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub selection_fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub cursor_bg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub cursor_fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub gutter_fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub gutter_active_fg: Color,
    #[serde(deserialize_with = "de_color_opt", serialize_with = "ser_color_opt")]
    pub gutter_bg: Option<Color>,
    #[serde(deserialize_with = "de_color_opt", serialize_with = "ser_color_opt")]
    pub line_indicator_bg: Option<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: None, // Transparent/Default
            fg: Color::Reset,
            border: Color::Grey,
            status_bg: Color::White,
            status_fg: Color::Black,
            accent: Color::Yellow,
            selection_bg: Color::Blue,
            selection_fg: Color::White,
            cursor_bg: Color::White,
            cursor_fg: Color::Black,
            gutter_fg: Color::Grey,
            gutter_active_fg: Color::Yellow,
            gutter_bg: None,
            line_indicator_bg: Some(Color::DarkGrey),
        }
    }
}

fn parse_color(s: &str) -> Result<Color, String> {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() != 6 {
            return Err(format!("Invalid hex color length: {}", s));
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        Ok(Color::Rgb { r, g, b })
    } else {
        match s.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "darkgrey" | "darkgray" => Ok(Color::DarkGrey),
            "red" => Ok(Color::Red),
            "darkred" => Ok(Color::DarkRed),
            "green" => Ok(Color::Green),
            "darkgreen" => Ok(Color::DarkGreen),
            "yellow" => Ok(Color::Yellow),
            "darkyellow" => Ok(Color::DarkYellow),
            "blue" => Ok(Color::Blue),
            "darkblue" => Ok(Color::DarkBlue),
            "magenta" => Ok(Color::Magenta),
            "darkmagenta" => Ok(Color::DarkMagenta),
            "cyan" => Ok(Color::Cyan),
            "darkcyan" => Ok(Color::DarkCyan),
            "white" => Ok(Color::White),
            "grey" | "gray" => Ok(Color::Grey),
            "reset" => Ok(Color::Reset),
            _ => Err(format!("Unknown color name: {}", s)),
        }
    }
}

fn de_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_color(&s).map_err(serde::de::Error::custom)
}

fn de_color_opt<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(s) if s.to_lowercase() == "reset" || s.to_lowercase() == "none" => Ok(None),
        Some(s) => parse_color(&s).map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub fn ser_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match color {
        Color::Reset => "reset".to_string(),
        Color::Black => "black".to_string(),
        Color::DarkGrey => "darkgrey".to_string(),
        Color::Red => "red".to_string(),
        Color::DarkRed => "darkred".to_string(),
        Color::Green => "green".to_string(),
        Color::DarkGreen => "darkgreen".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::DarkYellow => "darkyellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::DarkBlue => "darkblue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::DarkMagenta => "darkmagenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::DarkCyan => "darkcyan".to_string(),
        Color::White => "white".to_string(),
        Color::Grey => "grey".to_string(),
        Color::Rgb { r, g, b } => format!("#{:02x}{:02x}{:02x}", r, g, b),
        _ => "reset".to_string(),
    };
    serializer.serialize_str(&s)
}

pub fn ser_color_opt<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match color {
        Some(c) => ser_color(c, serializer),
        None => serializer.serialize_str("none"),
    }
}

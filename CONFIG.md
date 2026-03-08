# Configuration

Barcode can be configured using a `config.toml` file.

## Configuration Paths

The editor searches for configuration files in the following order:

1. Path specified via `--config-file <FILE>`
2. `BARCODE_CONFIG_DIR` environment variable
3. `$XDG_CONFIG_HOME/barcode/`
4. `~/.config/barcode/`

## `config.toml` Structure

### `status_line`

Controls the appearance and behavior of the status line at the bottom.

- `enabled` (bool, default: `true`): Whether to show the status line.
- `text_left`, `text_center`, `text_right` (string): Templates for the respective sections.
- `left_end` (string): Character for the left rounded edge.
- `right_end` (string): Character for the right rounded edge.

#### Formatting Variables

The following variables can be used in `text_left`, `text_center`, and `text_right`:

- `{mode}`: Current editor mode (NORMAL, INSERT, etc.).
- `{file}`: Current filename.
- `{dir}`: Current directory path
- `{time}`: Current time (HH:MM).
- `{date}`: Current date (YYYY-MM-DD).
- `{user}`: Current username.
- `{host}`: System hostname.
- `{line}`: Current line number (1-indexed).
- `{col}`: Current column number (1-indexed).
- `{cursor}`: Current cursor position as `line:col`.
- `{percent}`: Vertical progress through the file (e.g., `Top`, `Bot`, `50%`).

### `notification`

Controls floating notifications.

- `enabled` (bool, default: `true`): Whether notifications are shown.
- `h_anchor` (string): Horizontal position: `left` (`l`), `center` (`c`), or `right` (`r`).
- `v_anchor` (string): Vertical position: `top` (`t`), `center` (`c`), or `bottom` (`b`).
- `border_style` (string): `none`, `single`, `double`, or `rounded`.
- `timeout_secs` (int): How long notifications stay visible.

### `input`

Controls the command/input bar.

- `h_anchor`, `v_anchor` (string): Positioning: `left`, `center`, `right` and `top`, `center`, `bottom`.
- `border_style` (string): `none`, `single`, `double`, or `rounded`.

## Keymaps and Themes

The editor also looks for `keymap.toml` and `theme.toml` in the same configuration directories. These can be overridden via `--keymap-config` and `--theme-config` CLI arguments.

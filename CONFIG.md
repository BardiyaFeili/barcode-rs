# Configuration

Barcode can be configured using a `config.toml` file. This file allows you to
customize the editor's appearance and behavior to suit your workflow.

## Configuration Paths

The editor searches for configuration files in the following order:

1. Path specified via the `--config-file <FILE>` command-line argument.
2. The directory defined by the `BARCODE_CONFIG_DIR` environment variable.
3. The `$XDG_CONFIG_HOME/barcode/` directory.
4. The standard `~/.config/barcode/` directory.

## `config.toml` Structure

### `editor`

Global settings for the text editing experience.

- `margin` (int, default: `3`): The number of lines to maintain as a margin
  between the cursor and the top/bottom edges when scrolling.
- `wrap` (bool, default: `true`): If enabled, long lines will wrap to the
  next visual line instead of scrolling horizontally.

### `line_number`

Settings for the line number gutter.

- `mode` (string, default: `"relative"`): Line numbering mode. Options:
  `"none"`, `"absolute"`, or `"relative"`.
- `padding_left` (int, default: `3`): Space between the window edge and the
  line number.
- `padding_right` (int, default: `3`): Space between the line number and the
  text area.

### `status_line`

Controls the appearance and behavior of the status line at the bottom of the
screen.

- `enabled` (bool, default: `true`): Whether to show the status line.
- `text_left`, `text_center`, `text_right` (string): Templates defining the
  content of each respective section.
- `left_end` (string): Character used for the left rounded edge (e.g., ``).
- `right_end` (string): Character used for the right rounded edge (e.g., ``).

#### Formatting Variables

The following variables can be used within the status line templates:

- `{mode}`: Current editor mode (e.g., NORMAL, INSERT, COMMAND).
- `{file}`: Name of the current file.
- `{dir}`: Current directory path.
- `{time}`: Current system time (HH:MM).
- `{date}`: Current date (YYYY-MM-DD).
- `{user}`: Current username.
- `{host}`: System hostname.
- `{line}`: Current line number (1-indexed).
- `{col}`: Current column number (1-indexed).
- `{cursor}`: Current cursor position formatted as `line:col`.
- `{percent}`: Vertical progress through the file (e.g., `Top`, `Bot`, `50%`).

### `notification`

Settings for floating notifications.

- `enabled` (bool, default: `true`): Whether notifications are displayed.
- `h_anchor` (string): Horizontal position: `left` (`l`), `center` (`c`),
  or `right` (`r`).
- `v_anchor` (string): Vertical position: `top` (`t`), `center` (`c`),
  or `bottom` (`b`).
- `border_style` (string): Border type: `none`, `single`, `double`, or
  `rounded`.
- `timeout_secs` (int): Duration in seconds that notifications stay visible.

### `input`

Settings for the command/input bar.

- `h_anchor`, `v_anchor` (string): Positioning options similar to
  notifications.
- `border_style` (string): Border type: `none`, `single`, `double`, or
  `rounded`.

## Keymaps and Themes

Barcode also searches for `keymap.toml` and `theme.toml` in the same
configuration directories. You can override these via the `--keymap-config` and
`--theme-config` CLI arguments.

### `theme.toml` Structure

The theme file allows you to customize the color palette. Colors can be
specified by name (e.g., `"white"`, `"red"`, `"grey"`) or as hex codes
(e.g., `"#1a1b26"`).

- `bg` (string, default: `"reset"`): Main background color. Use `"reset"`
  or `"none"` for terminal transparency.
- `fg` (string, default: `"reset"`): Default text color.
- `border` (string, default: `"grey"`): Color for inactive window borders.
- `status_bg` (string, default: `"white"`): Background color of the status line

- `status_fg` (string, default: `"black"`): Foreground color of the status
  line.
- `accent` (string, default: `"yellow"`): Color for active window borders
  and other visual highlights.
- `selection_bg` (string, default: `"blue"`): Background color for text
  selection.
- `selection_fg` (string, default: `"white"`): Text color within a selection.
- `cursor_bg` (string, default: `"white"`): Color of the cursor block.
- `cursor_fg` (string, default: `"black"`): Text color under the cursor.
- `gutter_fg` (string, default: `"grey"`): Color of the line numbers in the
  gutter.
- `gutter_active_fg` (string, default: `"yellow"`): Color of the active line
  number.
- `gutter_bg` (string, default: `"reset"`): Background color of the gutter.

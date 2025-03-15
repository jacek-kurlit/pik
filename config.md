# Configuration

You can find default values below

```toml
# Size of the viewport
screen_size = { height = 20 } # run pik in 20 lines of the terminal
# screen_size = "fullscreen" # run pik in fullscreen

[ignore]
# ignore processes that path matches any of given regex
paths = []
# paths = ["/System/.*", "Applications/.*"]
# ignore other users processes
other_users = true
# ignore thread processes (on linux)
threads = true

### UI Configuration ###
[ui]
# Icons require nerd fonts v3
use_icons = false

[ui.process_table]
title = { alignment = "left", position = "top" }
border = { type = "rounded", style = { fg = "#60A5FA" } }

[ui.process_table.row]
selected_symbol = " "
even = { fg = "#E2E8F0", bg = "#0F172A" }
odd = { fg = "#E2E8F0", bg = "#020617" }
selected = { fg = "#60A5FA", add_modifier = "REVERSED" }

[ui.process_table.cell]
# normal = {}
highlighted = { bg = "Yellow", add_modifier = "ITALIC" }
```

## General options

| Field       | Description          | Possible values        |
| ----------- | -------------------- | ---------------------- |
| screen_size | Size of the viewport | fullscreen, height = n |

## Ignore filers

These properties are toml table under `[ignore]` section

| Field       | Description                                                                           | Possible values |
| ----------- | ------------------------------------------------------------------------------------- | --------------- |
| threads     | Ignore Linux threads                                                                  | true, false     |
| other_users | Ignore Other users processes                                                          | true, false     |
| paths       | List of path regex to ignore. If process matches any of the regex, it will be ignored | array of regex  |

Regex are defined using the [regex create](https://docs.rs/regex/latest/regex)

## UI & Theme

These properties are toml table under `[ui]` section

| Field     | Description            | Possible values |
| --------- | ---------------------- | --------------- |
| use_icons | Use Nerd font v3 icons | true, false     |

### Process table

These properties are toml table under `[ui.process_table]` section

| Field  | Description                | Possible values |
| ------ | -------------------------- | --------------- |
| title  | Title configuration        | See below       |
| border | Border configuration       | See below       |
| row    | Row styling configuration  | See below       |
| cell   | Cell styling configuration | See below       |

#### Title Configuration

These properties are toml table under `[ui.process_table.title]` section

| Field     | Description                 | Possible values           |
| --------- | --------------------------- | ------------------------- |
| alignment | Text alignment within title | "left", "center", "right" |
| position  | Position of the title       | "top", "bottom"           |

#### Border Configuration

These properties are toml table under `[ui.process_table.border]` section

| Field | Description                 | Possible values                                                            |
| ----- | --------------------------- | -------------------------------------------------------------------------- |
| type  | Style of border to display  | "plain", "rounded", "double", "thick", "quadrant_inside", "quadrant_outside" |
| style | Border color and formatting | Style configuration (see below)                                            |

#### Row Configuration

These properties are toml table under `[ui.process_table.row]` section

| Field           | Description                      | Possible values     |
| --------------- | -------------------------------- | ------------------- |
| even            | Style for even-numbered rows     | Style configuration |
| odd             | Style for odd-numbered rows      | Style configuration |
| selected        | Style for selected row           | Style configuration |
| selected_symbol | Symbol displayed on selected row | Any string          |

#### Cell Configuration

These properties are toml table under `[ui.process_table.cell]` section

| Field       | Description                 | Possible values     |
| ----------- | --------------------------- | ------------------- |
| normal      | Base style for cells        | Style configuration |
| highlighted | Style for highlighted cells | Style configuration |

#### Style Configuration

Styles can be configured with these properties:

| Field           | Description               | Possible values                                                                                         |
| --------------- | ------------------------- | ------------------------------------------------------------------------------------------------------- |
| fg              | Foreground color          | Color name or hex code (e.g., "#60A5FA")                                                                |
| bg              | Background color          | Color name or hex code                                                                                  |
| add_modifier    | Text modifiers to add     | "BOLD", "DIM", "ITALIC", "UNDERLINED", "SLOW_BLINK", "RAPID_BLINK", "REVERSED", "HIDDEN", "CROSSED_OUT" |
| sub_modifier    | Text modifiers to remove  | Same as add_modifier                                                                                    |
| underline_color | Color for underlined text | Color name or hex code                                                                                  |

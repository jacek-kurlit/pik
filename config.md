# Configuration

## General options

| Field       | Description          | Default     | Possible values        |
| ----------- | -------------------- | ----------- | ---------------------- |
| screen_size | Size of the viewport | height = 20 | fullscreen, height = n |

## Ignore filers

These properties are toml table under `[ignore]` section

| Field       | Description                                                                                | Default | Possible values |
| ----------- | ------------------------------------------------------------------------------------------ | ------- | --------------- |
| threads     | Ignore Linux threads                                                                       | true    | true, false     |
| other_users | Ignore Other users processes                                                               | true    | true, false     |
| paths       | List of path regex to ignore. If process matches at least of the regex, it will be ignored | empty   | array of regex  |

Regex are defined using the [regex create](https://docs.rs/regex/latest/regex)

## UI & Theme

These properties are toml table under `[ui]` section

| Field     | Description            | Default | Possible values |
| --------- | ---------------------- | ------- | --------------- |
| use_icons | Use Nerd font v3 icons | false   | true, false     |

### Process table

These properties are toml table under `[ui.process_table]` section

| Field  | Description                | Default                                            | Possible values |
| ------ | -------------------------- | -------------------------------------------------- | --------------- |
| title  | Title configuration        | `{ alignment = "Left", position = "Top" }`         | See below       |
| border | Border configuration       | `{ type = "Rounded", style = { fg = "#60A5FA" } }` | See below       |
| row    | Row styling configuration  | See below                                          | See below       |
| cell   | Cell styling configuration | See below                                          | See below       |

#### Title Configuration

These properties are toml table under `[ui.process_table.title]` section

| Field     | Description                 | Default | Possible values           |
| --------- | --------------------------- | ------- | ------------------------- |
| alignment | Text alignment within title | "Left"  | "Left", "Center", "Right" |
| position  | Position of the title       | "Top"   | "Top", "Bottom"           |

#### Border Configuration

These properties are toml table under `[ui.process_table.border]` section

| Field | Description                 | Default              | Possible values                                                            |
| ----- | --------------------------- | -------------------- | -------------------------------------------------------------------------- |
| type  | Style of border to display  | "Rounded"            | "Plain", "Rounded", "Double", "Thick", "QuadrantInside", "QuadrantOutside" |
| style | Border color and formatting | `{ fg = "#60A5FA" }` | Style configuration (see below)                                            |

#### Row Configuration

These properties are toml table under `[ui.process_table.row]` section

| Field           | Description                      | Default                                         | Possible values     |
| --------------- | -------------------------------- | ----------------------------------------------- | ------------------- |
| even            | Style for even-numbered rows     | `{ fg = "#E2E8F0", bg = "#0F172A" }`            | Style configuration |
| odd             | Style for odd-numbered rows      | `{ fg = "#E2E8F0", bg = "#020617" }`            | Style configuration |
| selected        | Style for selected row           | `{ fg = "#60A5FA", add_modifier = "REVERSED" }` | Style configuration |
| selected_symbol | Symbol displayed on selected row | " " (space)                                     | Any string          |

#### Cell Configuration

These properties are toml table under `[ui.process_table.cell]` section

| Field       | Description                 | Default                                      | Possible values     |
| ----------- | --------------------------- | -------------------------------------------- | ------------------- |
| normal      | Base style for cells        | `{}`                                         | Style configuration |
| highlighted | Style for highlighted cells | `{ bg = "Yellow", add_modifier = "ITALIC" }` | Style configuration |

#### Style Configuration

Styles can be configured with these properties:

| Field           | Description               | Default | Possible values                                                                                         |
| --------------- | ------------------------- | ------- | ------------------------------------------------------------------------------------------------------- |
| fg              | Foreground color          | None    | Color name or hex code (e.g., "#60A5FA")                                                                |
| bg              | Background color          | None    | Color name or hex code                                                                                  |
| add_modifier    | Text modifiers to add     | None    | "BOLD", "DIM", "ITALIC", "UNDERLINED", "SLOW_BLINK", "RAPID_BLINK", "REVERSED", "HIDDEN", "CROSSED_OUT" |
| sub_modifier    | Text modifiers to remove  | None    | Same as add_modifier                                                                                    |
| underline_color | Color for underlined text | None    | Color name or hex code                                                                                  |

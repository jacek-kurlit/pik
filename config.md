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
icons = "ascii" # nerd_font_v3 or custom (see below)

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

[ui.process_table.scrollbar]
# style = {}
# thumb_symbol = None
track_symbol = "│"
begin_symbol = "↑"
end_symbol = "↓"
margin = {horizontal = 0, vertical = 1}

[ui.process_details]
title = { alignment = "left", position = "top" }
border = { type = "rounded", style = {fg = "#60A5FA"}}

[ui.process_details.scrollbar]
# style = {}
# thumb_symbol = None
track_symbol = "│"
begin_symbol = "↑"
end_symbol = "↓"
margin = {horizontal = 0, vertical = 1}

[ui.search_bar]
# style = {}
cursor_style = {add_modifier = "REVERSED"}

[ui.popups]
border = {type = "rounded", style = {fg = "#4ade80"}}
selected_row = { bg = "#1e293b", add_modifier = "BOLD"}
primary = { fg = "#60A5FA" }
# secondary = {}
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

| Field           | Description                   | Possible values |
| --------------- | ----------------------------- | --------------- |
| icons           | Configure icons               | See below       |
| process_table   | Process Table Configuration   | See below       |
| process_details | Process Details Configuration | See below       |
| search_bar      | Search bar Configuration      | See below       |
| popups          | Popups Configuration          | See below       |

### Icons Configuration

By default no icons is configured which in equal to `ui.icons=ascii`
You can use nerd font icons by setting `ui.icons=nerd_font_v3`, this of course need nerd font installed
To set up your custom icons you may use this (nerd_font_v3 setup)

```toml
[ui.icons.custom]
user = "󰋦"
pid = ""
parent = "󱖁"
time = ""
cmd = "󱃸"
path = ""
args = "󱃼"
ports = ""
search_prompt = ""
```

### Process table

These properties are toml table under `[ui.process_table]` section

| Field     | Description                     | Possible values |
| --------- | ------------------------------- | --------------- |
| title     | Title configuration             | See below       |
| border    | Border configuration            | See below       |
| row       | Row styling configuration       | See below       |
| cell      | Cell styling configuration      | See below       |
| scrollbar | Scrollbar styling configuration | See below       |

### Process details

These properties are toml table under `[ui.process_details]` section

| Field     | Description                     | Possible values |
| --------- | ------------------------------- | --------------- |
| title     | Title configuration             | See below       |
| border    | Border configuration            | See below       |
| scrollbar | Scrollbar styling configuration | See below       |

### Search bar

These properties are toml table under `[ui.search_bar]` section

| Field        | Description                | Possible values |
| ------------ | -------------------------- | --------------- |
| style        | Style configuration        | See below       |
| cursor_style | Cursor style configuration | See below       |

### Popups

These properties are toml table under `[ui.popups]` section

| Field        | Description          | Possible values |
| ------------ | -------------------- | --------------- |
| border       | Border configuration | See below       |
| selected_row | Style configuration  | See below       |
| primary      | Style configuration  | See below       |
| secondary    | Style configuration  | See below       |

#### Title Configuration

Title can be configured with these properties:

| Field     | Description                 | Possible values           |
| --------- | --------------------------- | ------------------------- |
| alignment | Text alignment within title | "left", "center", "right" |
| position  | Position of the title       | "top", "bottom"           |

#### Border Configuration

Border can be configured with these properties:

| Field | Description                 | Possible values                                                              |
| ----- | --------------------------- | ---------------------------------------------------------------------------- |
| type  | Style of border to display  | "plain", "rounded", "double", "thick", "quadrant_inside", "quadrant_outside" |
| style | Border color and formatting | Style configuration (see below)                                              |

#### Row Configuration

Row can be configured with these properties:

| Field           | Description                      | Possible values     |
| --------------- | -------------------------------- | ------------------- |
| even            | Style for even-numbered rows     | Style configuration |
| odd             | Style for odd-numbered rows      | Style configuration |
| selected        | Style for selected row           | Style configuration |
| selected_symbol | Symbol displayed on selected row | Any string          |

#### Cell Configuration

Cell can be configured with these properties:

| Field       | Description                 | Possible values     |
| ----------- | --------------------------- | ------------------- |
| normal      | Base style for cells        | Style configuration |
| highlighted | Style for highlighted cells | Style configuration |

#### Scrollbar Configuration

Scrollbar can be configured with these properties:

| Field  | Description              | Possible values      |
| ------ | ------------------------ | -------------------- |
| style  | Base style for scrollbar | Style configuration  |
| thumb  | Thumb symbol             | String               |
| track  | Track symbol             | String               |
| begin  | Begin symbol             | String               |
| end    | End symbol               | String               |
| margin | Margin for scrollbar     | Margin configuration |

<--▮------->
^ ^ ^ ^
│ │ │ └ end
│ │ └──── track
│ └──────── thumb
└─────────── begin

#### Style Configuration

Styles can be configured with these properties:

| Field           | Description               | Possible values                                                                                         |
| --------------- | ------------------------- | ------------------------------------------------------------------------------------------------------- |
| fg              | Foreground color          | Color name or hex code (e.g., "#60A5FA")                                                                |
| bg              | Background color          | Color name or hex code                                                                                  |
| add_modifier    | Text modifiers to add     | "BOLD", "DIM", "ITALIC", "UNDERLINED", "SLOW_BLINK", "RAPID_BLINK", "REVERSED", "HIDDEN", "CROSSED_OUT" |
| sub_modifier    | Text modifiers to remove  | Same as add_modifier                                                                                    |
| underline_color | Color for underlined text | Color name or hex code                                                                                  |

#### Margin Configuration

Margin can be configured with these properties:

| Field      | Description                        | Possible values |
| ---------- | ---------------------------------- | --------------- |
| vertical   | Vertical margin (top and bottom)   | u16             |
| horizontal | Horizontal margin (left and right) | u16             |

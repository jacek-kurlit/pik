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

### Key mappings
[key_mappings]
next_item = ["down","tab", "ctrl+j", "ctrl+n"]
previous_item = ["up", "shift+tab", "ctrl+k", "ctrl+p"]
jump_ten_next_items = ["pagedown"]
jump_ten_previous_items = ["pageup"]
go_to_first_item = ["ctrl+up", "ctrl+home"]
go_to_last_item = ["ctrl+down", "ctrl+end"]

close = ["esc"]
quit = ["ctrl+c"]

kill_process = ["ctrl+x"]
refresh_process_list = ["ctrl+r"]
copy_process_pid = ["ctrl+y"]

scroll_process_details_down = ["ctrl+f"]
scroll_process_details_up = ["ctrl+b"]

select_process_parent = ["alt+p"]
select_process_family = ["alt+f"]
select_process_siblings = ["alt+s"]

toggle_help = ["ctrl+h"]
toggle_fps = ["ctrl+."]

cursor_left = ["left"]
cursor_right = ["right"]
cursor_home = ["home"]
cursor_end = ["end"]
delete_char = ["backspace"]
delete_next_char = ["delete"]
delete_word = ["ctrl+w"]
delete_to_start = ["ctrl+u"]

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

## Key mappings

These properties are toml table under `[key_mappings]` section

| Action                      | Description                 | Possible values |
| :-------------------------- | :-------------------------- | :-------------- |
| close                       | Closes the current view     | Key binding     |
| quit                        | Quits the application       | Key binding     |
| kill_process                | Kills the selected process  | Key binding     |
| refresh_process_list        | Refreshes the process list  | Key binding     |
| copy_process_pid            | Copies selected process PID | Key binding     |
| scroll_process_details_down | Scrolls details down        | Key binding     |
| scroll_process_details_up   | Scrolls details up          | Key binding     |
| select_process_parent       | Selects parent process      | Key binding     |
| select_process_family       | Selects process family      | Key binding     |
| select_process_siblings     | Selects process siblings    | Key binding     |
| toggle_help                 | Toggles help display        | Key binding     |
| cursor_left                 | Moves cursor left           | Key binding     |
| cursor_right                | Moves cursor right          | Key binding     |
| cursor_home                 | Moves cursor to line start  | Key binding     |
| cursor_end                  | Moves cursor to line end    | Key binding     |
| delete_char                 | Deletes character           | Key binding     |
| delete_next_char            | Deletes next character      | Key binding     |
| delete_word                 | Deletes word                | Key binding     |
| delete_to_start             | Deletes to line start       | Key binding     |
| next_item                   | Jumps to next item             | Key binding     |
| previous_item               | Jumps to previous item         | Key binding     |
| jump_ten_next_items         | Jumps 10 items down         | Key binding     |
| jump_ten_previous_items     | Jumps 10 items up           | Key binding     |
| go_to_first_item            | Jumps to first item         | Key binding     |
| go_to_last_item             | Jumps to last item          | Key binding     |

### Key binding

Key mapping rules:
- You may define binding as single key mapping `action = "ctrl+x"` or array `action = "ctrl+x", "alt+x"`
- You can prefix mapping with modifier and '+' sign, allowed values are: "ctrl", "alt", "shift", "super", "hyper", "meta"
- You **may not** define mapping as single char like `action = c` but you may use any special key `action = tab`
- If key binding is assigned to more than **one** action validation error will rise

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

# Configuration

The full default configuration is defined in [`default_config.toml`](default_config.toml).

Pik embeds that file into the binary at compile time, and your local `config.toml` is deep-merged on top of it. This means you can override only the fields you care about, including nested UI fields such as scrollbar symbols, styles, and margins.

Example partial override:

```toml
[ui.process_details.scrollbar]
thumb_symbol = "T"
margin = { horizontal = 2 }
```

In that example, all omitted scrollbar fields still come from `default_config.toml`.

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

| Action                      | Description                                     | Possible values |
| :-------------------------- | :---------------------------------------------- | :-------------- |
| close                       | Closes the current view                         | Key binding     |
| quit                        | Quits the application                           | Key binding     |
| kill_process                | Gracefully kills the selected process (SIGTERM) | Key binding     |
| force_kill_process          | Forcefully kills the selected process (SIGKILL) | Key binding     |
| refresh_process_list        | Refreshes the process list                      | Key binding     |
| copy_process_pid            | Copies selected process PID                     | Key binding     |
| scroll_process_details_down | Scrolls details down                            | Key binding     |
| scroll_process_details_up   | Scrolls details up                              | Key binding     |
| select_process_parent       | Selects parent process                          | Key binding     |
| select_process_family       | Selects process family                          | Key binding     |
| select_process_siblings     | Selects process siblings                        | Key binding     |
| toggle_help                 | Toggles help display                            | Key binding     |
| toggle_debug                | Toggles debug display                           | Key binding     |
| cursor_left                 | Moves cursor left                               | Key binding     |
| cursor_right                | Moves cursor right                              | Key binding     |
| cursor_home                 | Moves cursor to line start                      | Key binding     |
| cursor_end                  | Moves cursor to line end                        | Key binding     |
| delete_char                 | Deletes character                               | Key binding     |
| delete_next_char            | Deletes next character                          | Key binding     |
| delete_word                 | Deletes word                                    | Key binding     |
| delete_to_start             | Deletes to line start                           | Key binding     |
| next_item                   | Jumps to next item                              | Key binding     |
| previous_item               | Jumps to previous item                          | Key binding     |
| jump_ten_next_items         | Jumps 10 items down                             | Key binding     |
| jump_ten_previous_items     | Jumps 10 items up                               | Key binding     |
| go_to_first_item            | Jumps to first item                             | Key binding     |
| go_to_last_item             | Jumps to last item                              | Key binding     |

### Key binding

Key mapping rules:

- You may define binding as single key mapping `action = "ctrl+x"` or array `action = ["ctrl+x", "alt+x"]`
- You can prefix mapping with modifier and '+' sign, allowed values are: "ctrl", "alt", "shift", "super", "hyper", "meta"
- You can combine multiple modifiers: `action = "ctrl+alt+h"` or `action = "ctrl+shift+h"`
- Multiple modifiers with multiple bindings: `action = ["ctrl+alt+h", "ctrl+shift+h"]`
- You **may not** define mapping as single char like `action = "c"` but you may use any special key `action = "tab"`
- If key binding is assigned to more than **one** action validation error will rise

**Examples:**

```toml
# Single binding
quit = "ctrl+c"

# Multiple bindings for the same action
toggle_help = ["ctrl+h", "f1"]

# Combined modifiers
toggle_debug = "ctrl+alt+d"

# Multiple bindings with combined modifiers
toggle_help = ["ctrl+alt+h", "ctrl+shift+h"]
```

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

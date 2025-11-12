<div align="center">

<h1>
  <span style="font-size: 80px;">Pik</span>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
  <img alt="Pik" height="80" src="logo_light.svg"/>
</picture>
</h1>

[![Build status](https://github.com/jacek-kurlit/pik/actions/workflows/on_merge.yml/badge.svg)](https://github.com/jacek-kurlit/pik/actions)
[![GitHub Release](https://img.shields.io/github/v/release/jacek-kurlit/pik)](https://github.com/jacek-kurlit/pik/releases/latest)

</div>

Process Interactive Kill is a command line tool that helps to find and kill process.
It works like pkill command but search is interactive.
![Example pik](docs/pik.png)

This tool is still under development

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Application configuration](#application-configuration)
  - [Multiple meta key names support](#multiple-meta-key-names-support)
  - [Readline style support](#readline-style-support)
- [Caveats](#caveats)
- [Development](#development)
  - [Supported Systems](#supported-systems)
  - [Setup](#setup)
  - [Building](#building)

## Features

Pik allows to **fuzzy** search processes by:

- Name - No prefix is required, just type process name, for example 'firefox'
  ![Example search by name](docs/search_by_name.gif)
- Cmd Path - Prefix search with '/', for example '/firefox'
  ![Example search by path](docs/search_by_path.gif)
- Arguments - Prefix search with '-' for example '-foo'. Please note that if you want to use this feature in cli you must add `--`, for example `pik -- -foo`
  ![Example search by argument](docs/search_by_arg.gif)
- Ports - Prefix search with ':' for example ':8080'
  ![Example search by port](docs/search_by_port.gif)
- Everywhere - Prefix search with '~' for example '~firefox'
  ![Example search everywhere](docs/search_everywhere.gif)
- Select exact process by id - Prefix with '!' for example '!1234'
- Select process family (process + it's children) - Prefix with '@' for example '@1234'

After selecting process you can kill it with Ctrl + X

## Installation

**[Archives of precompiled binaries for pik are available for Linux and macOS.](https://github.com/jacek-kurlit/pik/releases)**

On **Arch Linux**

```sh
pacman -S pik
```

On **Fedora**

```sh
sudo dnf copr enable rusty-jack/pik
sudo dnf install pik
```

On **Tumbleweed**

```sh
sudo zypper install pik
```

On **Gentoo**

It is available via `lamdness` overlay

```sh
sudo eselect repository enable lamdness
sudo emaint -r lamdness sync
sudo emerge -av sys-process/pik
```

With **[dra](https://github.com/devmatteini/dra)**

```sh
dra download --install jacek-kurlit/pik
```

If you're a **Rust programmer**, pik can be installed with `cargo`.

```sh
cargo install pik
```

Alternatively, one can use [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) to install a pik
binary directly from GitHub:

```sh
cargo binstall pik
```

## Configuration

### Application configuration

You may set your preferences in `config.toml` file located in:

| Platform | Config dir                                         |
| -------- | -------------------------------------------------- |
| Linux    | /home/_:username_/.config/pik                      |
| MacOS    | /Users/_:username_/Library/Application Support/pik |
| Windows  | C:\Users\\_:username_\AppData\Roaming\pik          |

All options are optional, if skipped default values will be used.
Most of config fields have cli arg equivalent. If both are set cli arg is preferred.
Run `pik -- --help` to see cli options
Please refer to [config](config.md) for more details how to configure options,theme and key mappings

### Multiple meta key names support

Pik supports combining multiple modifier keys (meta keys) in key bindings. You can use any combination of "ctrl", "alt", "shift", "super", "hyper", and "meta" modifiers together.

**Examples:**
```toml
# Single modifier
quit = "ctrl+c"

# Combined modifiers
toggle_help = "ctrl+alt+h"
toggle_debug = "ctrl+shift+d"

# Multiple bindings with different modifier combinations for the same action
toggle_help = ["ctrl+alt+h", "ctrl+shift+h", "f1"]
```

This allows for flexible keybinding configurations that can accommodate different user preferences and avoid conflicts with terminal or OS shortcuts.

### Readline style support

You may configure pik to use readline style key mappings. Here is example config snippet that you may add to your `config.toml` file to enable basic readline style editing:

```toml
cursor_left = ["left", "ctrl+b"]
cursor_right = ["right", "ctrl+f"]
cursor_home = ["home", "ctrl+a"]
cursor_end = ["end", "ctrl+e"]
cursor_word_left = ["alt+b"]
cursor_word_right = ["alt+f"]
delete_char = ["backspace", "ctrl+h"]
delete_next_char = ["delete", "ctrl+d"]
delete_word = ["ctrl+w"]
delete_next_word = ["alt+d"]
delete_to_start = ["ctrl+u"]
delete_to_end = ["ctrl+k"]
```

Notice that you may need to adjust the other key mappings as well to avoid conflicts, and some key combinations may not work depending on your terminal emulator.

## Caveats

- Process name on linux system it is not always exe name also it is limited to 15 chars
- In linux process may appear on list but you are not allowed to get information about ports it uses. In such situations you need to run pik with root privileges

## Development

### Supported Systems

In theory pik is using coross compliant lib that allows to run it on all major platforms.
In pratice I'm using linux and development is performed based on this OS.
Pik will probably work on MacOs and Windows but that must be tested by community since I don't own computers with these OS'es.
If you are able to test it on windows or macos please create issue to let me know.

### Setup

- Rust 1.79+
- Cargo make [link](https://github.com/sagiegurari/cargo-make)
- Cargo nextest [link](https://github.com/nextest-rs/nextest)
- VHS [link](https://github.com/charmbracelet/vhs)

### Building

```sh
git clone https://github.com/jacek-kurlit/pik
cd pik
cargo build --release
./target/release/pik --version
```

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

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Caveats](#caveats)
- [Development](#development)

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

You may set your preferences in `config.toml` file located in `~/.config/pik` directory.
All options are optional, if skipped default values will be used.

You can find default values below

``` toml
# Size of the viewport
screen_size = { height = 20 } # run pik in 20 lines of the terminal
# screen_size = "fullscreen" # run pik in fullscreen

# Icons require nerd fonts v3
use_icons = false
```

### Key maps

| Key(s)                     | Action                 |
| -------------------------- | ---------------------- |
| `Esc` \| `Ctrl + C`        | Quit                   |
| `Ctrl + X`                 | Kill process           |
| `Ctrl + R`                 | Refresh processes list |
| `Ctrl + F`                 | Details forward        |
| `Ctrl + B`                 | Details backward       |
| `Tab` \| `Shift + Tab`     | Select next/previous   |
| `Arrow Down` \| `Arrow Up` | Select next/previous   |
| `Ctrl + J` \| `Ctrl + K`   | Select next/previous   |
| `Ctrl + Arrow Down` \| `Ctrl + Arrow Up`   | Select last/first   |

## Caveats

- Process name on linux system it is not always exe name also it is limited to 15 chars
- In linux process may appear on list but you are not allowed to get information about ports it uses. In such situations you need to run pik with root privileges
- Currently fuzzy search for args is not supported due to weird behavior - some processes pass all arguments as single causing them to always appear on list. Due to this fact args search is done by **contains** method

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

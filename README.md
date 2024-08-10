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

This tool is still under heavy development

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Caveats](#caveats)
- [Development](#development)

## Features

Pik allows to search processes by:

- Name - No prefix is required, just type process name, for example 'firefox'
![Example search by name](docs/search_by_name.gif)
- Cmd Path - Prefix search with '/', for example '/firefox'
![Example search by path](docs/search_by_path.gif)
- Arguments - Prefix search with '-' for example '-foo'. Please note that if you want to use this feature in cli you must add `--`, for example `pik -- -foo`
![Example search by argument](docs/search_by_arg.gif)
- Ports - Prefix search with ':' for example ':8080'
![Example search by port](docs/search_by_port.gif)

After selecting process you can kill it with Ctrl + X

## Installation

**[Archives of precompiled binaries for pik are available for Linux and macOS.](https://github.com/jacek-kurlit/pik/releases)**

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

### Key maps

- Esc - Quit
- Ctrl + X - Kill process
- Ctrl + F - Details forward
- Ctrl + B - Details backward
- Tab | Shift + Tab | Arrow Down | Arrow Up - Select next/previous

## Caveats

- Process name on linux system it is not always exe name also it is limited to 15 chars
- In linux process may appear on list but you are not allowed to get information about ports it uses. In such situations you need to run pik with root privileges

## Development

### Setup

- Rust 1.79+
- Cargo make [link](https://github.com/sagiegurari/cargo-make)
- Cargo nextest [link](https://github.com/nextest-rs/nextest)

### Building

```sh
git clone https://github.com/jacek-kurlit/pik
cd pik
cargo build --release
./target/release/pik --version
```

# Comparust

A simple Rust application with a GTK-based graphical interface, designed for macOS.
![](./screenshot.png)

## Features

- Compares folder contents, showing differences.
- Allows folder selection through UI or drag-and-drop.
- Filter and color-coded display options.

## Requirements

- Rust
- GTK 4 (Installable via Homebrew)

## Installation

Install GTK 4 via Homebrew:

``` brew install gtk4```

Add PKG_CONFIG_PATH for GTK libraries:

    export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/opt/homebrew/share/pkgconfig"

Clone Repository

git clone https://github.com/cccr/comparust.git
cd comparust

## Build and Bundle the App

Use cargo bundle to create a macOS .app package:

    cargo install cargo-bundle
    cargo bundle

The app bundle will be located in target/release/bundle/osz/comparust.app.

## Build for Windows

Follow https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html (Install GTK 4 with MSYS2 and the GNU toolchain) + replace with https://winlibs.com/#usage-commandprompt

## Running the App

To launch, double-click **comparust.app** in Finder. You can also run the binary directly:

    ./target/release/bundle/osz/comparust.app/Contents/MacOS/comparust

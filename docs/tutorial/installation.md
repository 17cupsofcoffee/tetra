# Installation

## Prerequisites

To get started with Tetra, you'll need a couple of things installed:

* Rust 1.32 or higher
* The SDL 2.0 development libraries
* The ALSA development libraries (only required on Linux)

Most of this is one-time setup, so let's get it out of the way!

### Installing Rust

Installing Rust is pretty simple - just go to [the website](https://www.rust-lang.org/tools/install) and download the Rustup toolchain manager.

Note that if you're developing on Windows with the default toolchain, you'll also need to install the [Microsoft Visual C++ Build Tools](https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2017), as Rust uses the MSVC linker when building.

### Installing SDL 2.0

Tetra uses SDL for windowing and input, so you will need to have both the runtime and development libraries installed.

> The instructions below are adapted from the README of the [sdl2](https://github.com/Rust-SDL2/rust-sdl2) crate - further information can be found there.

#### Windows

If you're using the default MSVC Rust toolchain:

1. Go to [the SDL website](https://www.libsdl.org/download-2.0.php) and download the Visual C++ version of the development libraries.
1. Copy the `.lib` files from the `SDL2-2.0.x/lib/x64` folder of the zip to the `%USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/lib` folder on your machine. If you are building on a beta/nightly toolchain, adjust the location accordingly.

If you're using the GNU-based Rust toolchain:

1. Go to [the SDL website](https://www.libsdl.org/download-2.0.php) and download the MinGW version of the development libraries.
1. Copy the `.lib` files from the `SDL2-2.0.x/x86_64-w64-mingw32/lib` folder of the zip to the `%USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib` folder on your machine. If you are building on a beta/nightly toolchain, adjust the location accordingly.

#### Mac

The easiest way to install SDL is via [Homebrew](http://brew.sh/):

```bash
brew install sdl2
```

You will also need to add the following to your `~/.bash_profile`, if it is not already present.

```bash
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
```

#### Linux

The SDL development libraries are distributed through most Linux package managers - here are a few examples:

```bash
# Ubuntu/Debian
sudo apt install libsdl2-dev

# Fedora/CentOS
sudo yum install SDL2-devel

# Arch Linux
sudo pacman -S sdl2
```

### Installing ALSA (Linux only)

On Linux, ALSA is used as the audio backend, so you will also need the ALSA development libraries installed. Similar to SDL, you can find these libraries on most Linux package managers:

```bash
# Ubuntu/Debian
sudo apt install libasound2-dev

# Fedora/CentOS
sudo yum install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

## Creating a New Project

Now that you've got the dependencies set up, we can start making a game!

First, create a new Cargo project:

```bash
cargo new --bin my-first-tetra-game
```

Then, add Tetra as a dependency in `Cargo.toml`:

```toml
[dependencies]
tetra = "0.2"
```

If you're on Windows, you'll need to place the SDL2 .dll in the root of your project (and alongside your .exe when distributing your game). You can download this from the ['runtime binaries' section of the SDL website](https://www.libsdl.org/download-2.0.php).

## Next Steps

Once those steps are complete, you're ready to [start writing your first game with Tetra](./getting-started.md)!

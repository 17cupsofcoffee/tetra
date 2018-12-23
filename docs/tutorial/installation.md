# Installation

## Creating a New Project

Create a new Cargo project:

```bash
cargo new --bin my-first-tetra-game
```

Then, add Tetra as a dependency in `Cargo.toml`:

```toml
[dependencies]
tetra = "0.2"
```

## Installing SDL 2.0

Tetra is built on top of SDL 2.0, so you will need to have both the runtime and development libraries installed.

The instructions below are adapted from the README of the [sdl2](https://github.com/Rust-SDL2/rust-sdl2) crate - further information can be found there.

### Windows (with MSVC toolchain)

1. Go to [the SDL website](https://www.libsdl.org/download-2.0.php) and download the Visual C++ version of the development libraries.
1. Copy the `.lib` files from the `SDL2-2.0.x/lib/x64` folder of the zip to the `%USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/lib` folder on your machine.
    * If you are building on a beta/nightly toolchain, adjust the location accordingly.
1. Copy `SDL2.dll` from the `SDL2-2.0.x/lib/x64` folder of the zip to the root of your Tetra project. You will also need to include this file alongside your `.exe` when distributing your game.

### Windows (with GNU toolchain)

1. Go to [the SDL website](https://www.libsdl.org/download-2.0.php) and download the MinGW version of the development libraries.
1. Copy the `.lib` files from the `SDL2-2.0.x/x86_64-w64-mingw32/lib` folder of the zip to the `%USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib` folder on your machine.
    * If you are building on a beta/nightly toolchain, adjust the location accordingly.
1. Copy `SDL2.dll` from the `SDL2-2.0.x/x86_64-w64-mingw32/bin` folder of the zip to the root of your Tetra project. You will also need to include this file alongside your `.exe` when distributing your game.

### Mac

The easiest way to install SDL is via [Homebrew](http://brew.sh/):

```bash
brew install sdl2
```

You will also need to add the following to your `~/.bash_profile`, if it is not already present.

```bash
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
```

### Linux

The SDL development libraries are distributed through most Linux package managers - here are a few examples:

#### Ubuntu/Debian

```bash
sudo apt-get install libsdl2-dev
```

#### Fedora/CentOS

```bash
sudo yum install SDL2-devel
```

#### Arch Linux

```bash
sudo pacman -S sdl2
```

## Next Steps

Once this is complete, you're ready to [start writing your first game with Tetra](/docs/getting-started)!
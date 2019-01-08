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

#### 

```bash
# Ubuntu/Debian
sudo apt-get install libsdl2-dev

# Fedora/CentOS
sudo yum install SDL2-devel

# Arch Linux
sudo pacman -S sdl2
```

### Advanced: Using a Bundled Version of SDL2

It's also possible to have your project automatically compile SDL2 from source as part of the build process. To do so, specify your dependency on Tetra like this:

```toml
[dependencies.tetra]
version = "0.2"
features = ["sdl2_bundled"]
```

This is more convienent, but does however require you to have various build tools installed on your machine (e.g. a C compiler, CMake, etc). In particular, this can be a pain on Windows - hence why it's not the default!

### Advanced: Static Linking

If you want to avoid your users having to install SDL2 themselves (or you having to distribute it as a DLL), you can specify for it to be statically linked:

```toml
[dependencies.tetra]
version = "0.2"
features = ["sdl2_static_link"]
```

This comes with some trade-offs, however - make sure you read [this document](https://hg.libsdl.org/SDL/file/default/docs/README-dynapi.md) in the SDL2 repository so that you understand what you're doing!

## Next Steps

Once this is complete, you're ready to [start writing your first game with Tetra](/docs/getting-started)!

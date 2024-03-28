# Installation

To get started with Tetra, you'll need several things installed, some of which are only needed on certain platforms:

* **All platforms:**
    * The latest stable version of Rust
    * The SDL 2.0 development libraries
* **Linux:**
    * The ALSA development libraries

## Installing Rust

Installing Rust is pretty simple - just go to [the website](https://www.rust-lang.org/tools/install) and download the Rustup toolchain manager.

Note that if you're developing on Windows with the default toolchain, you'll also need to install the [Microsoft Visual C++ Build Tools](https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2017). This is used for linking your code together.

## Installing SDL 2.0

Tetra uses a library called SDL for windowing and input, so you will need to have its runtime and development libraries installed in order for your project to compile.

### Windows

1. Go to [SDL's GitHub releases page](https://github.com/libsdl-org/SDL/releases) and download the version of the development libraries that corresponds to your Rust toolchain.
    * If you're using the MSVC toolchain, download `SDL2-devel-2.xx.x-VC.zip`.
    * If you're using the GNU toolchain, download `SDL2-devel-2.xx.x-mingw.zip`.
2. Inside the .zip file, open the `SDL2-2.xx.x/lib/x64` folder and extract `SDL2.lib` and `SDL2.dll` to the root of your Cargo workspace.
    * If you're on a 32-bit system, use the files in `SDL2-2.xx.x/lib/x86` instead.

You will also need to distribute `SDL2.dll` with your game - see the [distributing guide](./distributing.md) for more details.

You can also automatically compile SDL2 from source as part of the build process - see [Do I have to install SDL manually?](./faq.md/#do-i-have-to-install-sdl-manually) for more details. 

### Mac

The easiest way to install SDL is via [Homebrew](http://brew.sh/):

```bash
brew install sdl2
```

You will also need to add the following to your `~/.bash_profile`, if it is not already present.

```bash
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib:/usr/local/lib"
```

> [!WARNING]
> If you're building your game on Catalina, make sure that you use SDL 2.0.12 or higher - there is a
> [bug in earlier versions](https://hg.libsdl.org/SDL/rev/46b094f7d20e) which causes the OpenGL
> viewport to not scale correctly. See [issue #147](https://github.com/17cupsofcoffee/tetra/issues/147)
> for more information.

### Linux

The SDL development libraries are distributed through most Linux package managers - here are a few examples:

```bash
# Ubuntu/Debian
sudo apt install libsdl2-dev

# Fedora/CentOS
sudo yum install SDL2-devel

# Arch Linux
sudo pacman -S sdl2
```

## Installing ALSA (Linux only)

On Linux, ALSA is used as the audio backend, so you will need the ALSA development libraries installed. Similar to SDL, you can find these libraries on most Linux package managers:

```bash
# Ubuntu/Debian
sudo apt install libasound2-dev

# Fedora/CentOS
sudo yum install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

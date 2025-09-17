# Distributing

> [!NOTE]
> This page is a work in progress. It's especially missing information on Mac and Linux, as I'm not sure what the idiomatic ways of distributing games are on those platforms!
>
> If you have knowledge of these platforms, or other experience distributing games written in Rust, please [contribute](https://github.com/17cupsofcoffee/tetra-www/edit/main/docs/distributing.md)!

This page lists some of the things that should be taken into consideration when distributing a game built with Tetra.

## Required

### Build in Release Mode

By default, Cargo builds projects in debug mode, with very few optimizations. When you plan on distributing your game, you should make sure to run `cargo build` with the `--release` flag, to ensure that the final executable is as optimized as possible. There are [benchmarks in the FAQ](/faq/#benchmarks) which show that this makes a significant different to the performance!

### Include SDL

Tetra uses a C library called SDL 2.0 to interact with platform-specific functionality (such as windowing and input). Unlike Tetra's Rust dependencies, SDL is usually dynamically linked, meaning that the library needs to be present on the end user's machine for your application to run. Therefore, it is usually good practice to bundle SDL with your game when distributing it.

On Windows, the easiest way to do this is to include `SDL3.dll` in the same folder as your game's executable. You will probably have a copy of this file already if you followed the [installation guide](./installation.md), but if not, you can obtain it via [SDL's GitHub releases page](https://github.com/libsdl-org/SDL/releases).

Alternatively, you can choose to [statically link SDL into your game](/faq/#can-i-static-link-sdl) - however, this comes with [some tradeoffs](https://github.com/libsdl-org/SDL/blob/main/docs/README-dynapi.md) that need to be taken into account, so make sure you understand them before switching.

### Include Software Licenses

Tetra is provided under the terms of the [MIT License](https://opensource.org/licenses/MIT). One of the terms of this license is that you must include [the license text](https://github.com/17cupsofcoffee/tetra/blob/main/LICENSE) alongside 'all copies or substantial portions' of the library. Similar terms apply to many of Tetra's dependencies, including [the Rust standard library](https://github.com/rust-lang/rust/blob/master/COPYRIGHT), so it is important to make sure you've fulfilled these requirements when distributing your game to the public.

In practice, this usually means adding a screen to your game that displays open source licenses, or providing text files alongside the executable.

> [!TIP]
> [Embark Studios](https://www.embark-studios.com) has created a tool called [`cargo-about`](https://github.com/EmbarkStudios/cargo-about/) which can help you automate the arduous task of gathering these license files and outputting them into a template.
>
> Note, however, that it [does not currently provide license info for the Rust standard library](https://github.com/EmbarkStudios/cargo-about/issues/16) - you will need to obtain this yourself.

## Optional

### Change the Game's Icon/Metadata

By default, an application built by Cargo won't have any sort of icon or metadata, which can look somewhat unprofessional.

On Windows, you can add these via the [`embed-resource`](https://crates.io/crates/embed-resource) or [`winres`](https://github.com/mxre/winres) crates, which can be used via a `build.rs` script in your project. Alternatively, you can call directly into the `rc` command line tool included in the Windows SDK, or use an GUI application such as [ResEdit](http://www.resedit.net/).

On Mac, icons and metadata can be added by creating an Application Bundle, also known as a `.app`. Full details on the structure of these bundles can be found in [Apple's documentation](#change-the-game-s-icon-metadata).

On Linux, icons and metadata are generally provided via a [`.desktop` file](https://specifications.freedesktop.org/desktop-entry-spec/latest/) placed in `/usr/share/applications/` (for everyone on the machine) or `~/.local/share/applications` (for a single user). This will also make your game appear in the user's 'Applications' list.

> [!TIP]
> You may also want to consider using something like [AppImage](https://appimage.org/) to package your game and its metadata for distribution - this works in a similar manner to creating an Application Bundle on Mac.

### Hide the Console Window

On some platforms, applications built with Rust will display a console window while running by default. This can be useful while debugging, but is usually undesirable for the final release. 

On Windows, you can hide this window by adding the `windows_subsystem` attribute to your `main.rs`:

```rust
// To hide the console in all builds:
#![windows_subsystem = "windows"]

// To hide the console in release builds only:
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

> [!WARNING]
> When `windows_subsystem = "windows"` is applied, your application will no longer be able to read from `stdin` or write to `stdout`/`stderr`, as Windows does not attach them by default for GUI applications. Amonst other things, this means that you cannot log errors via `println!` or by returning them from `main` - no output will be displayed, even if the game is run from a command line.
>
> Make sure you have some other way of logging out fatal errors, otherwise it will be very difficult to diagnose the cause of crashes!

On Mac, packaging your game as an Application Bundle (`.app`) as described in ['Change the Game's Icon/Metadata'](#change-the-game-s-icon-metadata) will prevent the terminal window from being displayed.

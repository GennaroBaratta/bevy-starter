# Bevy Starter

This repo is a minimal starter for Bevy `0.19`

## Inspiration

- [`bevy_space`](https://github.com/perlindgren/bevy-space)
- [`bevy_new_2d`](https://github.com/TheBevyFlock/bevy_new_2d)
- [`sobevy`](https://codeberg.org/doomy/sobevy)
- [`Mischief in miniature`](https://github.com/alice-i-cecile/mischief-in-miniature)

## Building

You can build your game

```
cargo run
```

Native development features are enabled by default.

### Android development

One-time tools:

```powershell
rustup target add aarch64-linux-android
cargo install cargo-ndk cargo-watch
```

Connect one Android device with USB debugging enabled, then run:

```powershell
.\android-dev.ps1
```

With Android's **Wait for Debugger** enabled, start the watcher and JDB together:

```powershell
.\android-dev.ps1 -Debugger
```

The first build is slow. After that, `cargo-watch` watches `src`, `assets`, and
the Cargo manifests; each change rebuilds with `cargo-ndk`, installs the debug
APK, and launches it on the connected device. Press `Ctrl+C` to stop watching.

Depending on if you are building a 2D or 3D game you can set your Bevy features
accordingly in `Cargo.toml` to reduce compile times. For 2D games you can use:

```toml
bevy = { version = "0.19", default-features = false, features = ["2d"] }
```

See
[Cargo Feature Collections](https://bevy.org/news/bevy-0-18/#cargo-feature-collections)
for more information.

## Features

- Cargo configured according to Bevy guide with build optimizations
- [Avian](https://github.com/Jondolf/avian) physics
- Generic set of starting plugins with your games logic inside `GamePlugin`
- `TLDR.md` for quick reference and passing to LLMs
- `.agents` folder with bevy skills for LLMs

## Missing

- Deployment

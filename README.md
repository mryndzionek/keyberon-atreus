# `keyberon-atreus`

> [Keyberon](https://github.com/TeXitoi/keyberon) configuration
for my Atreus-like keyboards ([generator](https://github.com/mryndzionek/kbdSVGGen),
[first build](https://gist.github.com/mryndzionek/0fb397242e55262d831ccf3e8f38dcb0))
) using [`probe-run`] + [`defmt`] + [`flip-link`]

[`probe-run`]: https://crates.io/crates/probe-run
[`defmt`]: https://github.com/knurling-rs/defmt
[`flip-link`]: https://github.com/knurling-rs/flip-link

## Dependencies

#### 1. `flip-link`:

```console
$ cargo install flip-link
```

#### 2. `probe-run`:

```console
$ # make sure to install v0.2.0 or later
$ cargo install probe-run
```

## Build

#### Building for debug probe

Make sure to copy `memory.x.debug` to `memory.x` and then:

```console
cargo build --bin keyboard
```

or

```console
cargo run --bin keyboard
```

#### Building release version for bootloader

Make sure to copy `memory.x.bootlader` to `memory.x` and then:

```console
cargo objcopy --bin keyboard --release -- -O binary keyboard.bin
```


# tetron

A simple 2D game engine written in Rust. Think LÖVE but batteries-included and
ECS-oriented.

## Features

- [ ] Physics system
- [ ] 2D tilemap support
- [ ] [Rune](https://rune-rs.github.io) scripting API
- [ ] Dialogue system
- [ ] GUI framework (using microui)
- [x] First-class overlayfs-based modding support
  - load games from disk during development, then ship as a zip file!
- [x] Built-in scene support
- [ ] Browser support using WASM

## Using

Download the latest release from the [Releases](releases) page. To run a game:

```sh
tetron run --game /path/to/game
```

Mods can be specified by providing `--layer` with a path to a game file.

### Creating a game

```sh
tetron init my-game
```

This creates a simple Tetron hello world project that you can edit.

## LICENSE

Copyright &copy; 2025- Siddharth Singh, [The MIT License](./LICENSE.md).

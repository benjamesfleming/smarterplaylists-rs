# smarterplaylists-rs

Clone of the popular Spotify automatation tool [Smarter Playlists](http://smarterplaylists.playlistmachinery.com/), but with the backend written in Rust.

**Why?**

Why not? I started this clone to learn Rust, that is all.

## Features

- [ ] ...

## Installation

You can download the latest release binary from [here](https:://github.com/benjamesleming/smarterplaylists-rs/releases), use Docker, or build directly from the source code.

**Docker (WIP):**

```yaml
---
version: "3.0"
services:
  ...
````

**Build from source:**

> Note: Please ensure you have an up-to-date version of Rust installed

```bash
git clone https://github.com/benjamesfleming/smarterplaylists-rs
cd smarterplaylists-rs
cargo install --locked --path .
````

This will install the `smarterplaylists-rs` binary to `$HOME/.cargo/bin`.

## Development

### Git Hooks

This repository uses Git hooks to ensure code quality. The hooks are stored in the `.githooks` directory and are set up with:

```bash
git config core.hooksPath .githooks
```

Available hooks:
- `pre-commit`: Runs `cargo test` before allowing a commit to proceed

## License

MIT License

Copyright (c) 2023 Ben Fleming

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
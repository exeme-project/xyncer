# xyncer

[![MIT license](https://img.shields.io/badge/License-MIT-blue.svg)](https://lbesson.mit-license.org/)
[![Release Downloads](https://img.shields.io/github/downloads/exeme-project/xyncer/total.svg)](https://github.com/exeme-project/xyncer/releases)

- [xyncer](#xyncer)
  - [Installation](#installation)
    - [Using pre-built binaries](#using-pre-built-binaries)
    - [Running from source](#running-from-source)
  - [Contributing](#contributing)
  - [Acknowledgements](#acknowledgements)

xyncer (pronounced as _**/ˈzɪn.kər/**_) makes it easy to run Windows apps on Linux, as if they were native applications. 

> [!CAUTION]
> Xyncer is in **pre-alpha**. I aim to have a alpha build out within around a week, but for now proceed with caution.

## Installation

### Using pre-built binaries

Pre-built binaries are made available for every `x.x` release. If you want more frequent updates, then [run from source](#running-from-source). Download the binary for your OS from the [latest release](https://github.com/exeme-project/xyncer/releases/latest). There are quick links at the top of every release for popular OSes.

> [!IMPORTANT]\
> If you are on **Linux or macOS**, you may have to execute **`chmod +x path_to_binary`** in a shell to be able to run the binary.

### Running from source

> [!TIP]
> Use this method if none of the pre-built binaries work on your system, or if you want more frequent updates.

1. Make sure you have [Rust](https://rust-lang.org) installed. If you do not have rust installed, you can install it from [here](https://rustup.rs/).
2. Download and extract the repository from [here](https://github.com/exeme-project/xyncer/archive/refs/heads/master.zip). Alternatively, you can clone the repository with [Git](https://git-scm.com/) by running `git clone https://github.com/exeme-project/xyncer` in a terminal.
3. Navigate into the `/src` directory of your clone of this repository.
4. Run the command `cargo build --release`.
5. The compiled binary is in the `target/release` directory, named `main.exe` if you are on Windows, else `main`.

## Contributing

To learn more about contributing to The Exeme Language, please read the [**Contributing Guide**](https://github.com/exeme-project/.github/blob/main/CONTRIBUTING.md). There are ways to contribute to The Exeme Language even if you don't know how to code. We look forward to your contributions! 🚀

## Acknowledgements

Xyncer is inspired by many projects, namely:

- [**Cassowary**](https://github.com/casualsnek/cassowary).
- [**WinApps**](https://github.com/winapps-org/winapps).

Xyncer is also made possible by the following projects:

- [**Fastwebsockets**](https://github.com/denoland/fastwebsockets) - A fast WebSocket implementation for Rust.
- [**Tokio**](https://tokio.rs/) - An asynchronous runtime for Rust.
- [**XCap**](https://crates.io/crates/xcap) - A cross-platform screen capture library.

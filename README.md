# File Dialog plugin

A simple nu plugin that invokes the native file dialog of the os through [native-dialog-rs](https://github.com/native-dialog-rs/native-dialog-rs).

Works on windows (tested) and should work on GNU/Linux and BSD Unix. Doesn't work on macOS because [UI elements can't be spawned on a non-main thread](https://github.com/native-dialog-rs/native-dialog-rs?tab=readme-ov-file#why-the-program-crashed-when-opening-a-dialog-on-macos).

## Installation
```sh
cargo install nu_plugin_file_dialog
plugin add ~/.cargo/bin/nu_plugin_file_dialog
```

## Usage

run `file-dialog --help` to get the parameters and examples.

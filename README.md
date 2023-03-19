# UniTAS set up tool
Tool for setting up BepInEx and UniTAS for the unity game of your choice

- Currently this is a command line tool, but GUI support is planned

# Usage

## Help
This is where you can find the help menu

I highly recommend you check this out to see what commands are available
```
unitas_setup_tool --help
```

You can check command usage with `--help` or `-h` like so
```
unitas_setup_tool <command> --help
```

## Install
You can use this command to install stable release of UniTAS and BepInEx
```
unitas_setup_tool install <game_dir>
```

## Installing nightly builds
You can use this command to install the latest nightly build of UniTAS and / or BepInEx
```
unitas_setup_tool install <game_dir> --branch <unitas_branch> --bepinex-branch <bepinex_branch>
```

# How to build
- Make sure you have [rustup](https://www.rust-lang.org/tools/install) installed on your system
- To build the release version of the tool, run `cargo build --release`
- Otherwise, to build the debug version, run `cargo build`
- The binary will be located in `target/<release/debug>` directory
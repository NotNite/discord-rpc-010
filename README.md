# discord-rpc-010

Discord Rich Presence for 010 Editor.

![A screenshot of discord-rpc-010 in action](https://namazu.photos/i/2vlia4nr.png)

## Features

- [x] 010 Editor photo
- [x] Time ~~wasted~~ elapsed
- [x] Current file
- [ ] File size
- [ ] Cursor position

## Installation

Note: This is built and tested for 010 Editor v14.0 (64-bit, Windows). This will not work on other operating systems. I do not know if it works in other 010 Editor versions.

- Download the latest release (or build it yourself).
- Insert the file next to `010Editor.exe` with the name `winmm.dll`.
- Optional: Create a `discord-rpc.toml` file next to `010Editor.exe` for configuration.

## Configuration

Values shown are the defaults.

```toml
show_filename = true
```

## How it works

010 Editor has no plugin API, so [proxy-dll](https://github.com/rinlovesyou/dll-proxy-rs) is used to load into 010 Editor's address space. A proxy DLL abuses the fact that Windows loads libraries from the working directory before system files. You can find a library that is loaded on startup, create your own that forwards all functions to the original one, and then insert your own code.

The currently opened file resides in memory as a static address to a QString, so [skidscan](https://github.com/williamvenner/skidscan) and [iced](https://github.com/icedland/iced) are used to resolve the address. [discord-rich-presence](https://github.com/sardonicism-04/discord-rich-presence) is used to communicate with the Discord client.

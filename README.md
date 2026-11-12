![Made with VHS](https://vhs.charm.sh/vhs-3tLiFnxbJjlnyxJmBDrrgv.gif)

Tonus is a minimal terminal music player designed for offline playback.
It was developed primarily for Linux. Other platforms are untested.

### Prerequisites 
Requires Rust version 1.85 or newer.

### Features
- Vim style navigation
- Custom themes
- Basic search
- Repeat mode
- Playback seeking
- Random playback

### Installation
```bash
cargo install --locked tonus
```

### Building from source
```bash
git clone https://github.com/lekspa/tonus
cargo build --release
```

### Running
To run the player, execute `tonus` in a shell.

### Keymaps
Tonus runs with a default set of keymaps.

#### General

| Key          | Description       |
| ------------ | ----------------- |
| `j`, `Down`  | Move down         |
| `k`, `Up`    | Move up           |
| `/`          | Search            |
| `q`          | Quit              |

#### Playback

| Key          | Description         |
| ------------ | ------------------- |
| `n`          | Play next track     |
| `p`          | Play previous track |
| `Enter`      | Play selected       |
| `Space`      | Toggle pause play   |
| `+`, `=`     | Increase volume     |
| `-`          | Decrease volume     |
| `s`          | Toggle shuffle      |
| `r`          | Toggle repeat       |
| `Ctrl-Right` | Seek forward        |
| `Ctrl-Left`  | Seek backwards      |

#### Search

| Key          | Description       |
| ------------ | ----------------- |
| `Esc`        | Reset and close   |
| `Enter`      | Confirm query     |
| `Left`       | Move cursor left  |
| `Right`      | Move cursor right |

### Configuration
The configuration file should be created at `$HOME/.config/tonus.toml`.
An example file can be found in `example` directory in this repository.

### Themes
Theme can be customized by changing the `theme` section of configuration file.
The value for each field should be specified as hex. Alternatively, the following
colors can used.

- `"reset"`
- `"black"`
- `"white"`
- `"red"`
- `"lightred"`
- `"green"`
- `"lightgreen"`
- `"yellow"`
- `"lightyellow"`
- `"blue"`
- `"lightblue"`
- `"magenta"`
- `"lightmagenta"`
- `"cyan"`
- `"lightcyan"`
- `"gray"`
- `"darkgray"`

| Field                | Description                     |
| -------------------- | ------------------------------- |
| background           | Primary background              |
| foreground           | Primary text color              |
| surface              | Secondary background            |
| selection_background | Selected row background         |
| selection_foreground | Selected row text color         |
| accent               | Accent color                    |
| on_accent            | Text color on accent background |
| success              | Success text color              |
| failure              | Failure text color              |

### CLI arguments
Tonus can be run with optional CLI arguments. Arguments have higher precedence over the configuration file.

| Argument          | Description             |
| ----------------- | ----------------------- |
| --music-directory | Path to music directory |

### Logging
Tonus will attempt to create `$HOME/.local/share/tonus/tonus.log` to store logs.
Only events with a level of `INFO` of higher will be logged by default.
To see all events, set `RUST_LOG` environment variable to `debug` or `trace`.


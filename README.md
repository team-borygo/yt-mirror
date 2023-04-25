# yt-mirror

Do you use YouTube to listen to music, and you save it in bookmarks? I do.
It would be cool to have all this music on your local PC as a backup or just offline access, wouldn't it?

`yt-mirror` parses your browser bookmarks, looks for YouTube URLs, downloads movies, and converts them to a music file.
It runs incrementally, so you can synchronize videos as they are added up - only new bookmarks will be downloaded.

**This tool is still in active development**

Currently it works only on Linux (dependency on `mv` command, and Linux directory structure for data location, and defaults).

## Supported browsers

- Firefox
- Chrome/Chromium/Edge (and possibly other Chromium-based browsers)

## Requirements

- ffmpeg
- yt-dlp

## Installation

Requires Cargo (Rust)

`cargo install yt-mirror`

## Usage

Start by running the command `yt-mirror` once, without any arguments.

```sh
yt-mirror
```

It should create default config in `~/.config/yt-mirror/config.toml`.
Inside, you should modify `target_dir` (where your music will be downloaded) and `bookmark_files` list (see [Bookmarks location](#bookmarks-location)). At least one `bookmarks_files` entry must be provided.

Example minimal config:

```toml
bookmark_files = ["~/.config/BraveSoftware/Brave-Browser/Default/Bookmarks"]
target_dir = "~/music"
```

Then prepare bookmarks to synchronize:

```sh
yt-mirror prepare
```

Then run synchronization:

```sh
yt-mirror synchronize
```

(you can quit synchronization by pressing CTRL+C or ESC)

To show failed synchronizations:

```sh
yt-mirror failed [-s/--short]
```

`-s`/`--short` prints only failed YouTube ids without any decorations

If synchronization fail you can always try synchronizing failed bookmarks using `-r`/`--retry`:

```sh
yt-mirror synchronize -r
```

### Filtering videos to download

Using `--filter` user can utilise full power of [youtube-dl --match-filter](https://github.com/ytdl-org/youtube-dl/blob/master/README.md#video-selection).
It is useful to skip some videos, like those annoying 10h music videos that are sometimes added to bookmarks.

If video is filtered its process will be marked as "skipped".

Example - to download only videos with duration lower than 1000s:

```
yt-mirror synchronize --filter "duration < 1000"
```

### Configuration

You can pass custom configuration file location to any command using `-c`/`--config` parameter:

```sh
yt-mirror prepare -c ~/my-config-location.toml
```

Example full config:

```toml
# List of bookmark files to prepare (and eventually synchronize)
bookmark_files = ["~/.config/BraveSoftware/Brave-Browser/Default/Bookmarks"]
# target_dir is where your files will be moved after downloading
target_dir = "~/music"
# data_dir is used to store process bookmarks, and for other persistent data
# default: $XDG_DATA_HOME/yt-mirror or ~/.local/share/yt-mirror
data_dir = "~/my-data-dir"
# tmp_dir is used as location for temporary files
# default: /tmp
tmp_dir = "~/my-tmp-dir"
```

## Bookmarks locations

. | Linux | Windows
--- | --- | ---
Firefox | `~/.mozilla/firefox/*/places.sqlite` | `%appdata%\Mozilla\Firefox\Profiles\*\places.sqlite`
Chrome | `~/.config/google-chrome/Default/Bookmarks` | `%appdata%\..\Local\Google\Chrome\User Data\Default\Bookmarks`
Chromium | `~/.config/chromium/Default/Bookmarks` | ?
Brave | `~/.config/BraveSoftware/Brave-Browser/Default/Bookmarks` | `%appdata%\..\Local\BraveSoftware\Brave-Browser\User Data\Default`
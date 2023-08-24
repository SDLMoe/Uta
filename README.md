# Uta

> 好きな歌を歌う 好きな歌を歌う 好きな歌を歌う

Download ttml file from Apple Music, and convert it to lrc file.

## Build

```bash
cargo build --release
# Install if you want
cargo install --path .
```

## Usage

```bash
Usage: uta [OPTIONS] --url <URL> --token <TOKEN>

Options:
  -u, --url <URL>      URL of the song or album
  -s, --syllable       Need syllable lyrics
  -l, --lrc            Convert to lrc
  -t, --token <TOKEN>  Apple media token [env: APPLE_MEDIA_TOKEN=]
  -h, --help           Print help
  -V, --version        Print version
```

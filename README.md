# video_loop

A small Rust CLI that loops a source video to a target duration without cropping. It uses ffmpeg via the ffmpeg-sidecar crate, attempts stream-copy for speed and quality, and falls back to re-encoding with safe padding when needed.

## Requirements
- Rust (cargo)
- Network access for initial ffmpeg auto-download (handled by ffmpeg-sidecar)

## Folder Structure
- Cargo.toml
- Cargo.lock
- README.md
- src/
  - main.rs

Notes:
- The repository does not include a sample input video. Provide your own input file via -i/--input or place one in the current directory and name it input.mov to use defaults.
- An output file is created in the current directory unless you provide a custom path.

## Build

```bash
cargo build --release
```

## Usage

Run with defaults (1 minute, output name derived from input, current directory):

```bash
cargo run --release
```

Flags:
- -i, --input <path>    Input video file path
- -t, --length <value>  Target length; supports:
  - seconds: 90 (plain number)
  - minutes: 1m, 5m
- -o, --output <path>   Output file path (optional)
- --threads <n>         Number of ffmpeg threads (defaults to CPU cores)

Examples:
- Loop to 5 minutes, custom output:

```bash
cargo run --release -- -i /path/to/input.mov -t 5m -o /path/to/out.mov
```

- Loop to 90 seconds, output name derived from input in current dir:

```bash
cargo run --release -- -i /path/to/input.mov -t 90
```

- Loop to 10 minutes using 16 threads:

```bash
cargo run --release -- -i /path/to/input.mov -t 10m --threads 16
```

Default Output Naming:
- If -o/--output is not provided, the tool uses:
  - <input_stem>_loop_<minutes>min.<ext>
  - Example: input.mov → input_loop_1min.mov (for 1 minute)

## Behavior and Quality
- Stream copy is attempted first for speed and to preserve original encoding.
- If stream copy cannot be used, the tool re-encodes:
  - Video: H.264 (libx264), preset veryfast, CRF 20
  - Audio: AAC 128k
  - Padding filter ensures no cropping and even dimensions for encoder:
    - pad=ceil(iw/2)*2:ceil(ih/2)*2:(ceil(iw/2)*2-iw)/2:(ceil(ih/2)*2-ih)/2
  - Pixel format yuv420p to improve compatibility
- Output is trimmed exactly to the requested duration.

## Verify Output

Check duration:

```bash
ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 <output_file>
```

Check dimensions:

```bash
ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of default=nw=1 <output_file>
```

## Notes
- If your input has unusual timebase or codecs, stream copy may not be possible; the fallback re-encode path will be used automatically.
- Ensure the input path is valid and accessible from the directory where you run the command.

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::download;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::str::FromStr;

fn main() -> ExitCode {
    download::auto_download().unwrap();
    let mut input = String::from("");
    let mut output: Option<String> = None;
    let mut target_seconds = 60.0;
    let mut threads: Option<usize> = None;
    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        if a == "--input" || a == "-i" {
            if let Some(v) = args.next() {
                input = v;
            }
            continue;
        }
        if a == "--output" || a == "-o" {
            if let Some(v) = args.next() {
                output = Some(v);
            }
            continue;
        }
        if a == "--length" || a == "-t" {
            if let Some(v) = args.next() {
                let s = v.to_lowercase();
                if s.ends_with('m') {
                    let n = &s[..s.len() - 1];
                    if let Ok(x) = f64::from_str(n) {
                        target_seconds = x * 60.0;
                    }
                } else if s.ends_with('s') {
                    let n = &s[..s.len() - 1];
                    if let Ok(x) = f64::from_str(n) {
                        target_seconds = x;
                    }
                } else if let Ok(x) = f64::from_str(&s) {
                    target_seconds = x;
                }
            }
            continue;
        }
        if a == "--threads" {
            if let Some(v) = args.next() {
                if let Ok(x) = usize::from_str(&v) {
                    threads = Some(x);
                }
            }
            continue;
        }
    }
    let out = match output {
        Some(p) => p,
        None => {
            let p = Path::new(&input);
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("mov");
            let mins = (target_seconds / 60.0).round() as u32;
            let name = format!("{}_loop_{}min.{}", stem, mins, ext);
            PathBuf::from(name).to_string_lossy().to_string()
        }
    };
    if !Path::new(input.as_str()).exists() {
        eprintln!("missing input");
        return ExitCode::from(1);
    }
    let probe = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(input.as_str())
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&probe.stdout).trim().to_string();
    let dur = f64::from_str(&s).unwrap_or(0.0);
    if dur <= 0.0 {
        eprintln!("invalid duration");
        return ExitCode::from(1);
    }
    let loops = (target_seconds / dur).ceil() as usize;
    let mut list = String::new();
    for _ in 0..loops {
        list.push_str(format!("file '{}'\n", input).as_str());
    }
    let list_path = "concat_list.txt";
    fs::write(list_path, list).unwrap();
    let mut reenc_child = FfmpegCommand::new()
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(list_path)
        .arg("-map")
        .arg("0:v:0")
        .arg("-map")
        .arg("0:a?")
        .arg("-t")
        .arg(format!("{}", target_seconds as u32))
        .arg("-movflags")
        .arg("+faststart")
        .arg("-vf")
        .arg("scale=w=1920:h=-2:force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2:(ceil(iw/2)*2-iw)/2:(ceil(ih/2)*2-ih)/2,setsar=1")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("superfast")
        .arg("-crf")
        .arg("26")
        .arg("-shortest")
        .arg("-max_muxing_queue_size")
        .arg("1024")
        .arg("-threads")
        .arg(threads.map(|t| t.to_string()).unwrap_or_else(|| "0".to_string()))
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("96k")
        .arg(out.as_str())
        .spawn()
        .unwrap();
    let reenc_status = reenc_child.wait().unwrap();
    let _ = fs::remove_file(list_path);
    if reenc_status.success() {
        ExitCode::SUCCESS
    } else {
        eprintln!("ffmpeg failed");
        ExitCode::from(reenc_status.code().unwrap_or(1) as u8)
    }
}

//! Tải model ONNX của openWakeWord khi thư mục model còn thiếu (lần đầu mở app).

use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};

use anyhow::{Context, Result, ensure};

const BASE_URL: &str = "https://github.com/dscripka/openWakeWord/releases/download/v0.5.1";

/// (tên file trên máy, tên file trên bản phát hành openWakeWord)
const FILES: [(&str, &str); 3] = [
    ("melspectrogram.onnx", "melspectrogram.onnx"),
    ("embedding_model.onnx", "embedding_model.onnx"),
    ("hey_jarvis.onnx", "hey_jarvis_v0.1.onnx"),
];

pub(super) fn missing(dir: &Path) -> bool {
    FILES.iter().any(|(local, _)| !dir.join(local).is_file())
}

/// Tải các file còn thiếu ở thread riêng; gửi `dir` khi đủ model.
pub(super) fn fetch_in_background(dir: PathBuf) -> mpsc::Receiver<PathBuf> {
    let (tx, rx) = mpsc::channel();
    let spawned = thread::Builder::new()
        .name("hub-wake-models".into())
        .spawn(move || {
            eprintln!("Đang tải model wake word vào {}", dir.display());
            match download(&dir, BASE_URL) {
                Ok(()) => {
                    let _ = tx.send(dir);
                }
                Err(error) => eprintln!("Wake word disabled: tải model thất bại: {error:#}"),
            }
        });
    if let Err(error) = spawned {
        eprintln!("Wake word disabled: không tạo được thread tải model: {error}");
    }
    rx
}

/// Tải từng file còn thiếu qua file `.part` rồi đổi tên, để không để lại file dở.
pub(super) fn download(dir: &Path, base_url: &str) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("tạo {}", dir.display()))?;
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()?;
    for (local, remote) in FILES {
        let path = dir.join(local);
        if path.is_file() {
            continue;
        }
        let response = client
            .get(format!("{base_url}/{remote}"))
            .send()
            .with_context(|| format!("tải {remote}"))?;
        ensure!(
            response.status().is_success(),
            "tải {remote}: HTTP {}",
            response.status()
        );
        let bytes = response.bytes().with_context(|| format!("tải {remote}"))?;
        let part = dir.join(format!("{local}.part"));
        fs::write(&part, &bytes).with_context(|| format!("ghi {}", part.display()))?;
        fs::rename(&part, &path).with_context(|| format!("ghi {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

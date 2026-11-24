use super::{FILES, download, missing};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

/// HTTP server tối giản: trả `body` cho đường dẫn có trong `ok`, còn lại 404.
/// Ghi lại các đường dẫn đã được hỏi.
fn serve(ok: &'static [&'static str]) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = requests.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut line).unwrap();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap() <= 2 {
                    break;
                }
            }
            let name = path.trim_start_matches('/').to_owned();
            seen.lock().unwrap().push(name.clone());
            let response = if ok.contains(&name.as_str()) {
                let body = format!("model:{name}");
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into()
            };
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (base, requests)
}

const ALL: &[&str] = &[
    "melspectrogram.onnx",
    "embedding_model.onnx",
    "alexa_v0.1.onnx",
];

#[test]
fn downloads_every_missing_model_under_its_local_name() {
    let dir = tempfile::tempdir().unwrap();
    let models = dir.path().join("models");
    let (base, _) = serve(ALL);

    assert!(missing(&models));
    download(&models, &base).unwrap();

    assert!(!missing(&models));
    let classifier = std::fs::read_to_string(models.join("alexa.onnx")).unwrap();
    assert_eq!(classifier, "model:alexa_v0.1.onnx");
}

#[test]
fn skips_models_that_are_already_present() {
    let dir = tempfile::tempdir().unwrap();
    for (local, _) in FILES {
        std::fs::write(dir.path().join(local), "kept").unwrap();
    }
    let (base, requests) = serve(ALL);

    download(dir.path(), &base).unwrap();

    assert!(requests.lock().unwrap().is_empty());
    let kept = std::fs::read_to_string(dir.path().join("alexa.onnx")).unwrap();
    assert_eq!(kept, "kept");
}

#[test]
fn failed_download_leaves_no_partial_model() {
    let dir = tempfile::tempdir().unwrap();
    let (base, _) = serve(&["melspectrogram.onnx", "embedding_model.onnx"]);

    assert!(download(dir.path(), &base).is_err());

    assert!(!dir.path().join("alexa.onnx").exists());
    assert!(!dir.path().join("alexa.onnx.part").exists());
    assert!(missing(dir.path()));
}

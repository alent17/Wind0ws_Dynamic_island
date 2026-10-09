// Identical opt-in measurement adapter, injected into both isolated builds.
// Buffer counters describe tracked buffers, not decoder scratch or GPU memory.
use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf, sync::{Mutex, OnceLock, atomic::{AtomicU64, Ordering}}, time::{Duration, SystemTime, UNIX_EPOCH}};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static LOG: Mutex<()> = Mutex::new(());
static DOWNLOAD: AtomicU64 = AtomicU64::new(0);
static DOWNLOAD_PEAK: AtomicU64 = AtomicU64::new(0);
static DECODED: AtomicU64 = AtomicU64::new(0);
static DECODED_PEAK: AtomicU64 = AtomicU64::new(0);
static ENCODED: AtomicU64 = AtomicU64::new(0);
static ENCODED_PEAK: AtomicU64 = AtomicU64::new(0);
static COVERS: OnceLock<Vec<Value>> = OnceLock::new();
pub fn cover_url(title: &str) -> Option<String> {
    fn key(value:&str)->String{value.chars().filter(|c|c.is_alphanumeric()).flat_map(|c|c.to_lowercase()).collect()}
    let expected=key(title);
    COVERS.get()?.iter().find(|row|row["title"].as_str().map(key).as_deref()==Some(&expected))?["localUrl"].as_str().map(str::to_owned)
}
pub fn song_metadata(title: &str) -> Option<Value> {
    fn key(value:&str)->String{value.chars().filter(|c|c.is_alphanumeric()).flat_map(|c|c.to_lowercase()).collect()}
    let expected=key(title);
    let row=COVERS.get()?.iter().find(|row|row["title"].as_str().map(key).as_deref()==Some(&expected))?;
    Some(json!({"id":row["id"].as_str()?.parse::<u64>().ok()?,"name":row["title"],"duration":row["durationMs"],"album":{"picUrl":row["localUrl"]},"mvid":0}))
}
pub struct Buffer { kind: &'static str, bytes: u64 }
impl Buffer {
    pub fn new(kind: &'static str, bytes: usize) -> Self {
        let (current, peak) = counters(kind);
        let bytes=bytes as u64; let next=current.fetch_add(bytes, Ordering::Relaxed)+bytes;
        peak.fetch_max(next, Ordering::Relaxed); Self {kind,bytes}
    }
}
fn counters(kind: &str) -> (&'static AtomicU64,&'static AtomicU64) {
    match kind {"download"=>(&DOWNLOAD,&DOWNLOAD_PEAK),"decoded"=>(&DECODED,&DECODED_PEAK),_=>(&ENCODED,&ENCODED_PEAK)}
}
impl Drop for Buffer { fn drop(&mut self){counters(self.kind).0.fetch_sub(self.bytes,Ordering::Relaxed);} }
pub fn decoded(image: &image::DynamicImage) -> Buffer {Buffer::new("decoded",image.as_bytes().len())}
pub fn input_file(url: &str, path: &std::path::Path) {
    use std::{io::Read, hash::Hasher};
    let Ok(mut file)=std::fs::File::open(path) else{return};
    let mut hash=std::collections::hash_map::DefaultHasher::new();let mut bytes=0u64;let mut buffer=[0u8;8192];
    loop { match file.read(&mut buffer){Ok(0)=>break,Ok(n)=>{hash.write(&buffer[..n]);bytes+=n as u64},Err(_)=>return} }
    record(json!({"kind":"download-input","url":url,"bytes":bytes,"hash64":format!("{:016x}",hash.finish())}));
}
pub fn record(value: Value) {
    let Some(directory)=DIRECTORY.get() else{return}; let Ok(_lock)=LOG.lock() else{return};
    if let Ok(mut file)=OpenOptions::new().create(true).append(true).open(directory.join("events.jsonl")) {
        let row=json!({"at":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),"event":value,"downloadBufferedBytesPeak":DOWNLOAD_PEAK.load(Ordering::Relaxed),"decodedBuffersTrackedBytesPeak":DECODED_PEAK.load(Ordering::Relaxed),"encodedBuffersTrackedBytesPeak":ENCODED_PEAK.load(Ordering::Relaxed)});
        let _=writeln!(file,"{}",row);
    }
}
#[tauri::command]
pub fn benchmark_c_enabled() -> bool {DIRECTORY.get().is_some()}
#[tauri::command]
pub fn benchmark_c_report(value: Value) {record(value);}
pub fn start(app: AppHandle) {
    let Ok(path)=std::env::var("ISLE_BENCH_DIR") else{return};
    let path=PathBuf::from(path);if fs::create_dir_all(&path).is_err(){return} let _=DIRECTORY.set(path.clone());
    if let Ok(body)=fs::read_to_string(path.join("frozen-covers.json")){if let Ok(rows)=serde_json::from_str::<Vec<Value>>(&body){let _=COVERS.set(rows);}}
    record(json!({"kind":"started","pid":std::process::id(),"debugAssertions":cfg!(debug_assertions)}));
    std::thread::spawn(move || {
        let mut previous=String::new();
        loop {
            if let Ok(text)=fs::read_to_string(path.join("command.json")) {
                if text!=previous {
                    if let Ok(command)=serde_json::from_str::<Value>(&text) {
                        previous=text;
                        let app=app.clone();
                        tauri::async_runtime::spawn(async move {
                            let action=command["action"].as_str().unwrap_or("");
                            let result=match action {
                                "open"=>crate::commands::open_floating_window(app.clone()).await,
                                "close"=>crate::commands::close_floating_window(app.clone()).await,
                                "mark"=>Ok(()),
                                _=>Err(crate::error::AppError::business(3999,"Unknown benchmark action")),
                            };
                            record(json!({"kind":"command","command":command,"ok":result.is_ok(),"error":result.err().map(|e|e.to_string()),"floatingExists":app.get_webview_window("floating_player").is_some()}));
                        });
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    });
}

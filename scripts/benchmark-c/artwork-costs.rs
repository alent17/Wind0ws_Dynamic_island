//! Controlled codec-cost diagnostic; not a whole-app performance benchmark.
use std::{borrow::Cow, fs, time::Instant};
mod error {
    #[derive(Debug)] pub struct AppError(pub String);
    pub type AppResult<T> = Result<T, AppError>;
    impl AppError {
        pub fn business(_:u32, message:impl Into<String>)->Self {Self(message.into())}
        pub fn lock(message:impl Into<String>)->Self {Self(message.into())}
        pub fn parse(message:impl Into<String>)->Self {Self(message.into())}
    }
    impl From<std::io::Error> for AppError {fn from(value:std::io::Error)->Self {Self(value.to_string())}}
}
#[path="../../src-tauri/src/services/image_budget.rs"] mod image_budget;
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let manifest:serde_json::Value=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let directory=std::path::Path::new(&args[1]).parent().unwrap().join("covers");
    let inputs:Vec<_>=manifest.as_array().unwrap().iter().map(|row|fs::read(directory.join(row["file"].as_str().unwrap())).unwrap()).collect();
    let mut rounds=Vec::new();
    for round in 0..3 {
        let mut old_bytes=0usize;let mut new_bytes=0usize;let mut borrowed=0usize;
        let start=Instant::now();
        for input in &inputs {old_bytes+=std::hint::black_box(image_budget::normalize(input).unwrap()).len();}
        let normalize_ms=start.elapsed().as_secs_f64()*1000.;
        let start=Instant::now();
        for input in &inputs {
            let prepared=image_budget::prepare_artwork(input).unwrap();
            borrowed+=usize::from(matches!(prepared.bytes,Cow::Borrowed(_)));
            new_bytes+=std::hint::black_box(prepared.bytes).len();
        }
        rounds.push(serde_json::json!({"round":round+1,"normalizeMs":normalize_ms,"prepareMs":start.elapsed().as_secs_f64()*1000.,"oldOutputBytes":old_bytes,"newOutputBytes":new_bytes,"borrowedInputs":borrowed,"inputs":inputs.len()}));
    }
    fs::write(&args[2],serde_json::to_vec_pretty(&serde_json::json!({"scope":"Native image library from Release dependencies; three serial passes of identical frozen inputs. Full bounded validation remains; not Renderer retention or app CPU attribution.","rounds":rounds})).unwrap()).unwrap();
}

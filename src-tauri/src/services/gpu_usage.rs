//! GPU Engine performance counters, aggregated by adapter/engine across processes.
use std::collections::HashMap;

#[repr(C)]
struct CounterValue { status: u32, value: f64 }
#[repr(C)]
struct CounterItem { name: *const u16, value: CounterValue }

// Signatures and layouts follow the Windows SDK PDH declarations.
#[link(name = "pdh")]
extern "system" {
    fn PdhOpenQueryW(source: *const u16, data: usize, query: *mut isize) -> u32;
    fn PdhAddEnglishCounterW(query: isize, path: *const u16, data: usize, counter: *mut isize) -> u32;
    fn PdhCollectQueryData(query: isize) -> u32;
    fn PdhGetFormattedCounterArrayW(counter: isize, format: u32, bytes: *mut u32, count: *mut u32, items: *mut CounterItem) -> u32;
    fn PdhCloseQuery(query: isize) -> u32;
}

pub(super) struct GpuUsage { query: isize, counter: isize }
impl GpuUsage {
    pub fn new() -> Option<Self> {
        let mut query = 0;
        if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } != 0 { return None; }
        let mut result = Self { query, counter: 0 };
        let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0".encode_utf16().collect();
        if unsafe { PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut result.counter) } != 0 { return None; }
        unsafe { PdhCollectQueryData(query); }
        Some(result)
    }

    pub fn sample(&mut self) -> Option<f32> {
        const DOUBLE: u32 = 512;
        const MORE_DATA: u32 = 2147485650;
        if unsafe { PdhCollectQueryData(self.query) } != 0 { return None; }
        let (mut bytes, mut count) = (0, 0);
        if unsafe { PdhGetFormattedCounterArrayW(self.counter, DOUBLE, &mut bytes, &mut count, std::ptr::null_mut()) } != MORE_DATA { return None; }
        for _ in 0..3 {
            if bytes == 0 || bytes > 16 * 1024 * 1024 { return None; }
            // PDH's array contains pointers and doubles, so retain 8-byte alignment.
            let mut buffer = vec![0u64; (bytes as usize).div_ceil(8)];
            let status = unsafe { PdhGetFormattedCounterArrayW(self.counter, DOUBLE, &mut bytes, &mut count, buffer.as_mut_ptr().cast()) };
            if status == MORE_DATA { continue; }
            if status != 0 || count as usize > buffer.len() * 8 / std::mem::size_of::<CounterItem>() { return None; }
            let start = buffer.as_ptr() as usize;
            let end = start + buffer.len() * 8;
            let items = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<CounterItem>(), count as usize) };
            let mut engines = HashMap::new();
            for item in items {
                if item.value.status > 1 || !item.value.value.is_finite() { continue; }
                let address = item.name as usize;
                if address < start || address >= end || address % 2 != 0 { continue; }
                let name = unsafe { std::slice::from_raw_parts(item.name, (end-address)/2) };
                let length = name.iter().position(|value| *value == 0)?;
                let name = String::from_utf16_lossy(&name[..length]);
                // Remove the process ID; each physical GPU engine is shared by processes.
                if let Some((_, engine)) = name.split_once("_luid_") {
                    *engines.entry(engine.to_owned()).or_insert(0.0f64) += item.value.value.max(0.0);
                }
            }
            return busiest_engine(engines.values().copied());
        }
        None
    }
}
fn busiest_engine(values: impl Iterator<Item = f64>) -> Option<f32> {
    values.reduce(f64::max).map(|value| value.clamp(0.0, 100.0) as f32)
}
impl Drop for GpuUsage {
    fn drop(&mut self) { unsafe { PdhCloseQuery(self.query); } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_busiest_engine_without_summing_independent_engines() {
        assert_eq!(busiest_engine([40.0, 70.0, 12.0].into_iter()), Some(70.0));
        assert_eq!(busiest_engine([140.0].into_iter()), Some(100.0));
        assert_eq!(busiest_engine([].into_iter()), None);
    }
}

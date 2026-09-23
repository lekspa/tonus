/// Formats duration in seconds to "mm:ss" format
pub fn duration(duration: u64) -> String {
    format!("{}:{:02}", duration / 60, duration % 60)
}

/// Formats elapsed time and total time to "0:00/0:00" format
pub fn progress(elapsed: u64, length: u64) -> String {
    let elapsed = duration(elapsed.into());
    let length = duration(length);
    format!("{elapsed}/{length}")
}

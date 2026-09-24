/// Formats duration in seconds to "mm:ss" format
pub fn duration(duration: u64) -> String {
    format!("{}:{:02}", duration / 60, duration % 60)
}

/// Formats duration in seconds to "1h 30m" format
pub fn total_duration(duration: u64) -> String {
    let minutes = (duration / 60) % 60;
    let hours = (duration / 60 / 60) % 60;

    match hours {
        0 => format!("{minutes:02}m"),
        _ => format!("{hours}h {minutes:02}m"),
    }
}

/// Formats elapsed time and total time to "0:00/0:00" format
pub fn progress(elapsed: u64, length: u64) -> String {
    let elapsed = duration(elapsed);
    let length = duration(length);
    format!("{elapsed}/{length}")
}

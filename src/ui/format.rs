use std::time::Duration;

/// Formats track duration to "mm:ss" format
pub fn track_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;
    format!("{minutes}:{seconds:02}")
}

/// Formats duration to "1 hour 30 minutes" format
pub fn total_duration(duration: Duration) -> String {
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 60 / 60;

    match hours {
        0 => format!("{minutes} minutes"),
        1 => format!("{hours} hour {minutes} minutes"),
        _ => format!("{hours} hours {minutes} minutes"),
    }
}

/// Formats duration to "1h 30m" format
pub fn total_duration_compact(duration: Duration) -> String {
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 60 / 60;

    match hours {
        0 => format!("{minutes}m"),
        _ => format!("{hours}h {minutes}m"),
    }
}

/// Formats elapsed time and total time to "0:00/0:00" format
pub fn progress(elapsed: Duration, length: Duration) -> String {
    let elapsed = track_duration(elapsed);
    let length = track_duration(length);
    format!("{elapsed}/{length}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_duration() {
        assert_eq!(track_duration(Duration::from_secs(100)), "1:40");
        assert_eq!(track_duration(Duration::from_secs(200)), "3:20");
        assert_eq!(track_duration(Duration::from_secs(0)), "0:00");
    }

    #[test]
    fn formats_total_duration() {
        assert_eq!(total_duration(Duration::from_mins(5)), "5 minutes");
        assert_eq!(total_duration(Duration::from_mins(50)), "50 minutes");
        assert_eq!(
            total_duration(Duration::from_mins(100)),
            "1 hour 40 minutes"
        );
        assert_eq!(
            total_duration(Duration::from_mins(200)),
            "3 hours 20 minutes"
        );
    }

    #[test]
    fn formats_total_duration_compact() {
        assert_eq!(total_duration_compact(Duration::from_mins(5)), "5m");
        assert_eq!(total_duration_compact(Duration::from_mins(50)), "50m");
        assert_eq!(total_duration_compact(Duration::from_mins(100)), "1h 40m");
        assert_eq!(total_duration_compact(Duration::from_mins(200)), "3h 20m");
    }

    #[test]
    fn formats_progress() {
        assert_eq!(
            progress(Duration::from_secs(100), Duration::from_secs(150)),
            "1:40/2:30"
        );
    }
}

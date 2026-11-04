use std::time::Duration;

/// Formats duration to "mm:ss" format
pub fn duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;
    format!("{minutes}:{seconds:02}")
}

/// Formats duration to "1h 30m" format
pub fn total_duration(duration: Duration) -> String {
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 60 / 60;

    match hours {
        0 => format!("{minutes:02} minutes"),
        1 => format!("{hours} h {minutes:02} minutes"),
        _ => format!("{hours} hrs {minutes:02} minutes"),
    }
}

/// Formats elapsed time and total time to "0:00/0:00" format
pub fn progress(elapsed: Duration, length: Duration) -> String {
    let elapsed = duration(elapsed);
    let length = duration(length);
    format!("{elapsed}/{length}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_duration() {
        assert_eq!(duration(Duration::from_secs(100)), "1:40");
        assert_eq!(duration(Duration::from_secs(200)), "3:20");
        assert_eq!(duration(Duration::from_secs(0)), "0:00");
    }

    #[test]
    fn formats_total_duration() {
        assert_eq!(total_duration(Duration::from_mins(5)), "05 minutes");
        assert_eq!(total_duration(Duration::from_mins(50)), "50 minutes");
        assert_eq!(total_duration(Duration::from_mins(100)), "1 h 40 minutes");
        assert_eq!(total_duration(Duration::from_mins(200)), "3 hrs 20 minutes");
    }

    #[test]
    fn formats_progress() {
        assert_eq!(
            progress(Duration::from_secs(100), Duration::from_secs(150)),
            "1:40/2:30"
        );
    }
}

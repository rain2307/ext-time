use anyhow::{self, Context};
use std::ops::Sub;
use time::{ext::NumericalDuration, Duration, Time};

/// Extension trait for Time struct providing additional utility methods
pub trait ExtTime {
    /// Format time as HH:MM, padding minutes with zero if needed
    ///
    /// # Example
    /// ```
    /// use time::macros::time;
    /// use ext_time::ExtTime;
    ///
    /// let t = time!(9:05);
    /// assert_eq!(t.to_shorten(), "9:05");
    /// ```
    fn to_shorten(&self) -> String;

    /// Parse time string in HH:MM format
    ///
    /// # Arguments
    /// * `time_str` - Time string in "HH:MM" format
    ///
    /// # Returns
    /// * `Ok(Time)` - Parsed time
    /// * `Err` - If parsing fails
    fn from_str(time_str: &str) -> anyhow::Result<Time>;

    /// Calculate duration between two times, handling cross-day scenarios
    ///
    /// # Arguments
    /// * `right` - The time to subtract from self
    ///
    /// # Returns
    /// Duration between times, always positive by adding 24 hours if needed
    fn sub_ext(&self, right: Time) -> Duration;

    /// Reset seconds to zero, keeping hours and minutes
    fn reset_minute(&self) -> anyhow::Result<Time>;

    /// Check if two times are in the same minute
    fn is_same_minute(&self, other: &Time) -> bool;

    /// Check if time is between start and end (inclusive)
    /// Handles cross-day ranges (e.g., 23:00 to 01:00)
    fn is_between(&self, start: Time, end: Time) -> bool;

    /// Add minutes to time, wrapping around midnight if needed
    fn add_minutes(&self, minutes: i64) -> Time;
}

impl ExtTime for Time {
    fn to_shorten(&self) -> String {
        format!("{}:{:02}", self.hour(), self.minute())
    }

    fn from_str(time_str: &str) -> anyhow::Result<Time> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                if hour < 24 && minute < 60 {
                    return Time::from_hms(hour, minute, 0)
                        .with_context(|| format!("Invalid time components: {}:{}", hour, minute));
                }
            }
        }

        anyhow::bail!(
            "Invalid time format. Expected HH:MM or H:MM, got: {}",
            time_str
        )
    }

    fn sub_ext(&self, right: Time) -> Duration {
        let diff = self.clone().sub(right);
        if diff.is_negative() {
            24.hours() + diff
        } else {
            diff
        }
    }

    fn reset_minute(&self) -> anyhow::Result<Time> {
        Time::from_hms(self.hour(), self.minute(), 0)
            .with_context(|| format!("Failed to reset seconds for time: {:?}", self))
    }

    fn is_same_minute(&self, other: &Time) -> bool {
        self.minute() == other.minute() && self.hour() == other.hour()
    }

    fn is_between(&self, start: Time, end: Time) -> bool {
        if start <= end {
            *self >= start && *self <= end
        } else {
            // Handle cross-day range (e.g., 23:00 to 01:00)
            *self >= start || *self <= end
        }
    }

    fn add_minutes(&self, minutes: i64) -> Time {
        let total_minutes = self.hour() as i64 * 60 + self.minute() as i64 + minutes;
        let normalized_minutes = total_minutes.rem_euclid(24 * 60);
        let hours = (normalized_minutes / 60) as u8;
        let minutes = (normalized_minutes % 60) as u8;
        Time::from_hms(hours, minutes, self.second()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::time;

    #[test]
    fn test_shorten() {
        let t = time!(22:01);
        assert_eq!(t.to_shorten(), "22:01");

        let t = time!(9:05);
        assert_eq!(t.to_shorten(), "9:05");
    }

    #[test]
    fn test_from_str() {
        let t = <Time as ExtTime>::from_str("9:30").unwrap();
        assert_eq!(t.hour(), 9);
        assert_eq!(t.minute(), 30);

        assert!(<Time as ExtTime>::from_str("25:00").is_err());
        assert!(<Time as ExtTime>::from_str("invalid").is_err());
    }

    #[test]
    fn test_sub_ext() {
        let t1 = time!(23:00);
        let t2 = time!(1:00);
        assert_eq!(t1.sub_ext(t2), Duration::hours(22));
        assert_eq!(t2.sub_ext(t1), Duration::hours(2));
    }

    #[test]
    fn test_is_between() {
        let t = time!(23:30);
        assert!(t.is_between(time!(23:00), time!(0:00)));

        let t = time!(0:30);
        assert!(t.is_between(time!(23:00), time!(1:00)));

        let t = time!(12:00);
        assert!(!t.is_between(time!(23:00), time!(1:00)));
    }

    #[test]
    fn test_add_minutes() {
        let t = time!(23:30);
        assert_eq!(t.add_minutes(40).to_shorten(), "0:10");

        let t = time!(12:00);
        assert_eq!(t.add_minutes(-30).to_shorten(), "11:30");
    }
}

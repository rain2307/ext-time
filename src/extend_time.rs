use std::ops::Sub;
use thiserror::Error;
use time::{Duration, Time, ext::NumericalDuration};

#[derive(Error, Debug)]
pub enum TimeError {
    #[error("Invalid time format. Expected HH:MM or H:MM, got: {0}")]
    InvalidFormat(String),
    #[error("Invalid time components: {0}:{1}")]
    InvalidComponents(u8, u8),
    #[error("Failed to reset seconds for time: {0:?}")]
    ResetSecondsError(Time),
    #[error("Invalid seconds value: {0}")]
    InvalidSeconds(i64),
    #[error("Invalid alignment unit: {0}")]
    InvalidAlignmentUnit(u64),
    #[error("Failed to add time: {0:?}")]
    AddTimeError(Time),
}

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
    fn from_str(time_str: &str) -> Result<Time, TimeError>;

    /// Calculate duration between two times, handling cross-day scenarios
    ///
    /// # Arguments
    /// * `right` - The time to subtract from self
    ///
    /// # Returns
    /// Duration between times, always positive by adding 24 hours if needed
    fn sub_ext(&self, right: Time) -> Duration;

    /// Reset seconds to zero, keeping hours and minutes
    fn reset_minute(&self) -> Result<Time, TimeError>;

    /// Check if two times are in the same minute
    fn is_same_minute(&self, other: &Time) -> bool;

    /// Check if time is between start and end (inclusive)
    /// Handles cross-day ranges (e.g., 23:00 to 01:00)
    fn is_between(&self, start: Time, end: Time) -> bool;

    /// Add minutes to time, wrapping around midnight if needed
    fn add_minutes(&self, minutes: i64) -> Time;

    /// Convert seconds (hours + minutes + seconds) to Time
    ///
    /// # Arguments
    /// * `seconds` - Total seconds (hours * 3600 + minutes * 60 + seconds)
    ///
    /// # Returns
    /// * `Ok(Time)` - Converted time
    /// * `Err` - If seconds value is invalid
    fn from_seconds(seconds: i64) -> Result<Time, TimeError>;

    /// Convert Time to seconds (hours + minutes + seconds)
    ///
    /// # Returns
    /// Total seconds (hours * 3600 + minutes * 60 + seconds)
    fn to_seconds(&self) -> i64;

    /// Align time to the nearest interval
    ///
    /// # Arguments
    /// * `interval` - Interval in seconds (can be negative for backward alignment)
    ///
    /// # Returns
    /// * `Ok(Time)` - Aligned time
    /// * `Err(Error)` - If interval is 0
    fn align_to(&self, interval: i64) -> Result<Time, TimeError>;

    /// Get next day at the same time
    fn next_day(&self) -> Time;

    /// Get next hour at the same minute and second
    fn next_hour(&self) -> Time;

    /// Get next minute at the same second
    fn next_minute(&self) -> Time;

    /// Get next second
    fn next_second(&self) -> Time;

    /// Convert time to seconds, ignoring minutes and seconds
    ///
    /// # Returns
    /// Total seconds of hours (hours * 3600)
    ///
    /// Note: Returns i64 to support time differences and negative values
    fn to_hour_seconds(&self) -> i64;

    /// Convert time to seconds, ignoring seconds
    ///
    /// # Returns
    /// Total seconds of hours and minutes (hours * 3600 + minutes * 60)
    ///
    /// Note: Returns i64 to support time differences and negative values
    fn to_minute_seconds(&self) -> i64;
}

impl ExtTime for Time {
    fn to_shorten(&self) -> String {
        format!("{}:{:02}", self.hour(), self.minute())
    }

    fn from_str(time_str: &str) -> Result<Time, TimeError> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                if hour < 24 && minute < 60 {
                    return Time::from_hms(hour, minute, 0)
                        .map_err(|_| TimeError::InvalidComponents(hour, minute));
                }
            }
        }

        Err(TimeError::InvalidFormat(time_str.to_string()))
    }

    fn sub_ext(&self, right: Time) -> Duration {
        let diff = self.clone().sub(right);
        if diff.is_negative() {
            24.hours() + diff
        } else {
            diff
        }
    }

    fn reset_minute(&self) -> Result<Time, TimeError> {
        Time::from_hms(self.hour(), self.minute(), 0)
            .map_err(|_| TimeError::ResetSecondsError(*self))
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

    fn from_seconds(seconds: i64) -> Result<Time, TimeError> {
        if seconds < 0 || seconds >= 24 * 3600 {
            return Err(TimeError::InvalidSeconds(seconds));
        }

        let hours = (seconds / 3600) as u8;
        let minutes = ((seconds % 3600) / 60) as u8;
        let secs = (seconds % 60) as u8;

        Time::from_hms(hours, minutes, secs)
            .map_err(|_| TimeError::InvalidComponents(hours, minutes))
    }

    fn to_seconds(&self) -> i64 {
        self.hour() as i64 * 3600 + self.minute() as i64 * 60 + self.second() as i64
    }

    fn align_to(&self, interval: i64) -> Result<Time, TimeError> {
        if interval == 0 {
            return Err(TimeError::InvalidAlignmentUnit(interval.abs() as u64));
        }

        let total_seconds = self.to_seconds();
        let aligned_seconds = (total_seconds / interval) * interval;

        Time::from_seconds(aligned_seconds).map_err(|_| TimeError::InvalidSeconds(aligned_seconds))
    }

    fn next_day(&self) -> Time {
        // Since Time doesn't have day concept, we just return the same time
        *self
    }

    fn next_hour(&self) -> Time {
        let next_hour = (self.hour() + 1) % 24;
        Time::from_hms(next_hour, self.minute(), self.second()).unwrap()
    }

    fn next_minute(&self) -> Time {
        if self.minute() == 59 {
            let next_hour = (self.hour() + 1) % 24;
            Time::from_hms(next_hour, 0, self.second()).unwrap()
        } else {
            Time::from_hms(self.hour(), self.minute() + 1, self.second()).unwrap()
        }
    }

    fn next_second(&self) -> Time {
        if self.second() == 59 {
            if self.minute() == 59 {
                let next_hour = (self.hour() + 1) % 24;
                Time::from_hms(next_hour, 0, 0).unwrap()
            } else {
                Time::from_hms(self.hour(), self.minute() + 1, 0).unwrap()
            }
        } else {
            Time::from_hms(self.hour(), self.minute(), self.second() + 1).unwrap()
        }
    }

    fn to_hour_seconds(&self) -> i64 {
        self.hour() as i64 * 3600
    }

    fn to_minute_seconds(&self) -> i64 {
        self.hour() as i64 * 3600 + self.minute() as i64 * 60
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

    #[test]
    fn test_from_seconds() {
        let t = <Time as ExtTime>::from_seconds(37230).unwrap(); // 10:20:30
        assert_eq!(t.hour(), 10);
        assert_eq!(t.minute(), 20);
        assert_eq!(t.second(), 30);

        let t = <Time as ExtTime>::from_seconds(3660).unwrap(); // 1:01:00
        assert_eq!(t.hour(), 1);
        assert_eq!(t.minute(), 1);
        assert_eq!(t.second(), 0);

        assert!(<Time as ExtTime>::from_seconds(-1).is_err());
        assert!(<Time as ExtTime>::from_seconds(24 * 3600).is_err());
    }

    #[test]
    fn test_to_seconds() {
        let t = time!(10:20:30);
        assert_eq!(t.to_seconds(), 37230);

        let t = time!(1:01:00);
        assert_eq!(t.to_seconds(), 3660);

        let t = time!(0:00:00);
        assert_eq!(t.to_seconds(), 0);

        let t = time!(23:59:59);
        assert_eq!(t.to_seconds(), 86399);
    }

    #[test]
    fn test_align_to() {
        // Test alignment to 5 minutes
        let t = time!(10:34:00);
        let aligned = t.align_to(300).unwrap(); // 5 minutes = 300 seconds
        assert_eq!(aligned.hour(), 10);
        assert_eq!(aligned.minute(), 30);
        assert_eq!(aligned.second(), 0);

        // Test alignment to 5 seconds
        let t = time!(00:00:03);
        let aligned = t.align_to(5).unwrap();
        assert_eq!(aligned.hour(), 0);
        assert_eq!(aligned.minute(), 0);
        assert_eq!(aligned.second(), 0);

        // Test alignment to 1 hour
        let t = time!(14:30:45);
        let aligned = t.align_to(3600).unwrap();
        assert_eq!(aligned.hour(), 14);
        assert_eq!(aligned.minute(), 0);
        assert_eq!(aligned.second(), 0);

        // Test invalid unit
        let t = time!(10:00:00);
        assert!(t.align_to(0).is_err());
        assert!(t.align_to(24 * 3600).is_err());
    }

    #[test]
    fn test_next_hour() {
        let t = time!(10:30:45);
        let next = t.next_hour();
        assert_eq!(next.hour(), 11);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);

        let t = time!(23:30:45);
        let next = t.next_hour();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_minute() {
        let t = time!(10:30:45);
        let next = t.next_minute();
        assert_eq!(next.hour(), 10);
        assert_eq!(next.minute(), 31);
        assert_eq!(next.second(), 45);

        let t = time!(10:59:45);
        let next = t.next_minute();
        assert_eq!(next.hour(), 11);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 45);

        let t = time!(23:59:45);
        let next = t.next_minute();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_second() {
        let t = time!(10:30:45);
        let next = t.next_second();
        assert_eq!(next.hour(), 10);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 46);

        let t = time!(10:30:59);
        let next = t.next_second();
        assert_eq!(next.hour(), 10);
        assert_eq!(next.minute(), 31);
        assert_eq!(next.second(), 0);

        let t = time!(10:59:59);
        let next = t.next_second();
        assert_eq!(next.hour(), 11);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);

        let t = time!(23:59:59);
        let next = t.next_second();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn test_to_hour_seconds() {
        let t = time!(10:20:30);
        assert_eq!(t.to_hour_seconds(), 36000); // 10 * 3600

        let t = time!(0:30:45);
        assert_eq!(t.to_hour_seconds(), 0);

        let t = time!(23:59:59);
        assert_eq!(t.to_hour_seconds(), 82800); // 23 * 3600
    }

    #[test]
    fn test_to_minute_seconds() {
        let t = time!(10:20:30);
        assert_eq!(t.to_minute_seconds(), 37200); // 10 * 3600 + 20 * 60

        let t = time!(0:30:45);
        assert_eq!(t.to_minute_seconds(), 1800); // 30 * 60

        let t = time!(23:59:59);
        assert_eq!(t.to_minute_seconds(), 86340); // 23 * 3600 + 59 * 60
    }
}

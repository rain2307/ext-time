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


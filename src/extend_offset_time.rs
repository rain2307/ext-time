use thiserror::Error;
use time::{
    Duration, OffsetDateTime, Time, UtcOffset,
    format_description::{self},
    macros::format_description as fd,
};

#[derive(Error, Debug)]
pub enum OffsetDateTimeError {
    #[error("Invalid offset hours: {0}")]
    InvalidOffsetHours(i8),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Invalid milliseconds: {0}")]
    InvalidMilliseconds(u16),
    #[error("Failed to parse datetime: {0}")]
    ParseError(String),
    #[error("Failed to format datetime: {0}")]
    FormatError(String),
    #[error("Invalid seconds value: {0}")]
    InvalidSeconds(i64),
    #[error("Invalid alignment unit: {0}")]
    InvalidAlignmentUnit(u64),
    #[error("Failed to add time: {0:?}")]
    AddTimeError(OffsetDateTime),
}

pub trait ExtOffsetDateTime {
    /// Check if two timestamps are in the same minute
    fn is_same_minute(&self, b: &OffsetDateTime) -> bool;

    /// Reset seconds and subseconds to zero
    fn reset_minute(&self) -> OffsetDateTime;

    /// Get timestamp in milliseconds
    fn milli_timestamp(&self) -> i64;

    /// Format datetime to display string with timezone
    fn to_display_string(&self, offset_hours: i8) -> String;

    /// Format datetime to Chinese style string with timezone
    fn to_chinese_string(&self) -> String;

    /// Parse timestamp in milliseconds with timezone offset (hours from UTC)
    fn from_milliseconds(
        timestamp: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Parse timestamp in seconds with timezone offset (hours from UTC)
    fn from_seconds(
        timestamp: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Parse datetime from date string, time string and milliseconds with timezone
    fn from_date_time(
        date: &str,
        time: &str,
        milli: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Parse datetime from simple format string (YYYYMMDD_HHMM) with timezone
    fn from_simple(dt: &str, offset_hours: i8) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Convert date format from YYYYMMDD to YYYY.MM.DD
    fn convert_to_dot_date(input: &str) -> Result<String, OffsetDateTimeError>;

    /// Get current time with specified timezone offset (hours from UTC)
    fn now_with_offset(offset_hours: i8) -> OffsetDateTime {
        OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(offset_hours, 0, 0).unwrap())
    }

    /// Replace time part with seconds (hours + minutes + seconds)
    ///
    /// # Arguments
    /// * `seconds` - Total seconds (hours * 3600 + minutes * 60 + seconds)
    ///
    /// # Returns
    /// * `Ok(OffsetDateTime)` - DateTime with new time part
    /// * `Err` - If seconds value is invalid
    fn replace_time_with_seconds(
        &self,
        seconds: i64,
    ) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Align time to the nearest interval
    ///
    /// # Arguments
    /// * `interval` - Interval in seconds (can be negative for backward alignment)
    ///
    /// # Returns
    /// * `Ok(OffsetDateTime)` - Aligned time
    /// * `Err(Error)` - If interval is 0
    fn align_to(&self, interval: i64) -> Result<OffsetDateTime, OffsetDateTimeError>;

    /// Get next day at the same time
    fn next_day(&self) -> OffsetDateTime;

    /// Get next hour at the same minute and second
    fn next_hour(&self) -> OffsetDateTime;

    /// Get next minute at the same second
    fn next_minute(&self) -> OffsetDateTime;

    /// Get next second
    fn next_second(&self) -> OffsetDateTime;

    /// Convert time part to seconds, ignoring minutes and seconds
    ///
    /// # Returns
    /// Total seconds of hours (hours * 3600)
    ///
    /// Note: Returns i64 to support time differences and negative values
    fn to_hour_seconds(&self) -> i64;

    /// Convert time part to seconds, ignoring seconds
    ///
    /// # Returns
    /// Total seconds of hours and minutes (hours * 3600 + minutes * 60)
    ///
    /// Note: Returns i64 to support time differences and negative values
    fn to_minute_seconds(&self) -> i64;

    /// Calculate duration from current time to specified target time
    ///
    /// # Arguments
    /// * `target_hour` - Target hour (0-23)
    /// * `target_minute` - Target minute (0-59)
    /// * `target_second` - Target second (0-59)
    ///
    /// # Returns
    /// Duration from current time to target time, handling cross-day scenarios
    ///
    /// # Example
    /// ```
    /// use ext_time::ExtOffsetDateTime;
    /// use time::OffsetDateTime;
    ///
    /// let now = OffsetDateTime::now_utc();
    /// let duration = now.duration_to_time(20, 0, 0); // Duration to 20:00:00
    /// ```
    fn duration_to_time(&self, target_hour: u8, target_minute: u8, target_second: u8) -> Duration;
}

impl ExtOffsetDateTime for OffsetDateTime {
    fn is_same_minute(&self, b: &OffsetDateTime) -> bool {
        self.hour() == b.hour() && self.minute() == b.minute()
    }

    fn reset_minute(&self) -> OffsetDateTime {
        let time = Time::from_hms(self.hour(), self.minute(), 0).expect("Invalid time components");
        self.replace_time(time)
    }

    fn milli_timestamp(&self) -> i64 {
        (self.unix_timestamp() as i64) * 1000 + self.millisecond() as i64
    }

    fn to_display_string(&self, offset_hours: i8) -> String {
        let offset = UtcOffset::from_hms(offset_hours, 0, 0).expect("Invalid offset hours");
        self.to_offset(offset)
            .format(
                &format_description::parse(
                    "[year]-[month]-[day] [hour repr:24]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
                )
                .unwrap(),
            )
            .expect("Failed to format datetime")
    }

    fn to_chinese_string(&self) -> String {
        let offset = UtcOffset::from_hms(8, 0, 0).expect("Invalid offset hours");
        let format = format_description::parse(
            "[year]年[month]月[day]日 [hour]时[minute]分[second]秒 [offset_hour sign:mandatory]:[offset_minute]",
        )
        .expect("parse");
        self.to_offset(offset)
            .format(&format)
            .expect("Failed to format datetime")
    }

    fn from_milliseconds(
        timestamp: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError> {
        let seconds = timestamp / 1000;
        let millis = timestamp % 1000;
        let offset = UtcOffset::from_hms(offset_hours, 0, 0)
            .map_err(|_| OffsetDateTimeError::InvalidOffsetHours(offset_hours))?;

        let dt = OffsetDateTime::from_unix_timestamp(seconds as i64)
            .map_err(|_| OffsetDateTimeError::InvalidTimestamp(seconds as i64))?;

        let dt = dt
            .replace_millisecond(millis as u16)
            .map_err(|_| OffsetDateTimeError::InvalidMilliseconds(millis as u16))?;

        Ok(dt.to_offset(offset))
    }

    fn from_seconds(
        timestamp: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError> {
        let offset = UtcOffset::from_hms(offset_hours, 0, 0)
            .map_err(|_| OffsetDateTimeError::InvalidOffsetHours(offset_hours))?;

        let dt = OffsetDateTime::from_unix_timestamp(timestamp as i64)
            .map_err(|_| OffsetDateTimeError::InvalidTimestamp(timestamp as i64))?;

        Ok(dt.to_offset(offset))
    }

    fn from_date_time(
        date_str: &str,
        time_str: &str,
        milli: u64,
        offset_hours: i8,
    ) -> Result<OffsetDateTime, OffsetDateTimeError> {
        let format = fd!(
            "[year][month][day] [hour]:[minute]:[second].[subsecond digits:3] [offset_hour \
             sign:mandatory]:[offset_minute]:[offset_second]"
        );
        let dt = format!(
            "{} {}.{:03} {:+03}:00:00",
            date_str, time_str, milli, offset_hours
        );
        OffsetDateTime::parse(&dt, &format)
            .map_err(|e| OffsetDateTimeError::ParseError(e.to_string()))
    }

    fn from_simple(dt: &str, offset_hours: i8) -> Result<OffsetDateTime, OffsetDateTimeError> {
        let format = fd!("[year][month][day]_[hour][minute] [offset_hour sign:mandatory]");
        let dt = format!("{} {:+03}", dt, offset_hours);
        OffsetDateTime::parse(&dt, &format)
            .map_err(|e| OffsetDateTimeError::ParseError(e.to_string()))
    }

    fn convert_to_dot_date(input: &str) -> Result<String, OffsetDateTimeError> {
        let parse_format = fd!("[year][month][day]");
        let date = time::Date::parse(input, &parse_format)
            .map_err(|e| OffsetDateTimeError::ParseError(e.to_string()))?;

        let output_format = fd!("[year].[month].[day]");
        date.format(&output_format)
            .map_err(|e| OffsetDateTimeError::FormatError(e.to_string()))
    }

    fn replace_time_with_seconds(
        &self,
        seconds: i64,
    ) -> Result<OffsetDateTime, OffsetDateTimeError> {
        if seconds < 0 || seconds >= 24 * 3600 {
            return Err(OffsetDateTimeError::InvalidSeconds(seconds));
        }

        let hours = (seconds / 3600) as u8;
        let minutes = ((seconds % 3600) / 60) as u8;
        let secs = (seconds % 60) as u8;

        let time = Time::from_hms(hours, minutes, secs)
            .map_err(|_| OffsetDateTimeError::InvalidSeconds(seconds))?;

        Ok(self.replace_time(time))
    }

    fn align_to(&self, interval: i64) -> Result<OffsetDateTime, OffsetDateTimeError> {
        if interval == 0 {
            return Err(OffsetDateTimeError::InvalidAlignmentUnit(
                interval.abs() as u64
            ));
        }

        let total_seconds =
            self.hour() as i64 * 3600 + self.minute() as i64 * 60 + self.second() as i64;
        let aligned_seconds = (total_seconds / interval) * interval;

        let hours = (aligned_seconds / 3600) as u8;
        let minutes = ((aligned_seconds % 3600) / 60) as u8;
        let secs = (aligned_seconds % 60) as u8;

        let time = Time::from_hms(hours, minutes, secs)
            .map_err(|_| OffsetDateTimeError::InvalidSeconds(aligned_seconds))?;

        Ok(self.replace_time(time))
    }

    fn next_day(&self) -> OffsetDateTime {
        self.clone() + Duration::days(1)
    }

    fn next_hour(&self) -> OffsetDateTime {
        self.clone() + Duration::hours(1)
    }

    fn next_minute(&self) -> OffsetDateTime {
        self.clone() + Duration::minutes(1)
    }

    fn next_second(&self) -> OffsetDateTime {
        self.clone() + Duration::seconds(1)
    }

    fn to_hour_seconds(&self) -> i64 {
        self.hour() as i64 * 3600
    }

    fn to_minute_seconds(&self) -> i64 {
        self.hour() as i64 * 3600 + self.minute() as i64 * 60
    }

    fn duration_to_time(&self, target_hour: u8, target_minute: u8, target_second: u8) -> Duration {
        // Create target time in the same date and timezone as current time
        let target_time = Time::from_hms(target_hour, target_minute, target_second)
            .expect("Invalid target time components");

        // Create target datetime for today
        let target_today = self.replace_time(target_time);

        // Calculate duration to target time today
        let duration_to_today = target_today - *self;

        if duration_to_today.is_positive() || duration_to_today.is_zero() {
            // Target time is later today
            duration_to_today
        } else {
            // Target time is tomorrow (cross-day scenario)
            let target_tomorrow = target_today + Duration::days(1);
            target_tomorrow - *self
        }
    }
}

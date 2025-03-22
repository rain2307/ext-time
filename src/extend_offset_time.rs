use thiserror::Error;
use time::{
    Date, Duration, Month, OffsetDateTime, Time, UtcOffset,
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

    /// Align time part to the specified unit
    ///
    /// # Arguments
    /// * `unit_seconds` - The unit to align to in seconds (e.g., 300 for 5 minutes, 5 for 5 seconds)
    ///
    /// # Returns
    /// * `Ok(OffsetDateTime)` - DateTime with aligned time part
    /// * `Err` - If unit is invalid (must be positive and less than 24 hours)
    fn align_time_to(&self, unit_seconds: u64) -> Result<OffsetDateTime, OffsetDateTimeError>;

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
    fn to_hour_seconds(&self) -> i64;

    /// Convert time part to seconds, ignoring seconds
    ///
    /// # Returns
    /// Total seconds of hours and minutes (hours * 3600 + minutes * 60)
    fn to_minute_seconds(&self) -> i64;
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

    fn align_time_to(&self, unit_seconds: u64) -> Result<OffsetDateTime, OffsetDateTimeError> {
        if unit_seconds == 0 || unit_seconds >= 24 * 3600 {
            return Err(OffsetDateTimeError::InvalidAlignmentUnit(unit_seconds));
        }

        let total_seconds =
            self.hour() as i64 * 3600 + self.minute() as i64 * 60 + self.second() as i64;
        let aligned_seconds = (total_seconds / unit_seconds as i64) * unit_seconds as i64;

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
}

#[allow(dead_code)]
pub trait ExtendOffsetTime {
    fn start_of_day(&self) -> Self;
    fn end_of_day(&self) -> Self;
    fn start_of_week(&self) -> Self;
    fn end_of_week(&self) -> Self;
    fn start_of_month(&self) -> Self;
    fn end_of_month(&self) -> Self;
}

impl ExtendOffsetTime for OffsetDateTime {
    fn start_of_day(&self) -> Self {
        self.replace_time(Time::from_hms(0, 0, 0).unwrap())
    }

    fn end_of_day(&self) -> Self {
        self.replace_time(Time::from_hms(23, 59, 59).unwrap())
    }

    fn start_of_week(&self) -> Self {
        let days_from_monday = self.weekday().number_days_from_monday();
        self.start_of_day() - Duration::days(days_from_monday as i64)
    }

    fn end_of_week(&self) -> Self {
        let days_to_sunday = 6 - self.weekday().number_days_from_monday();
        self.end_of_day() + Duration::days(days_to_sunday as i64)
    }

    fn start_of_month(&self) -> Self {
        let date = Date::from_calendar_date(self.year(), self.month(), 1).unwrap();
        date.with_time(Time::from_hms(0, 0, 0).unwrap())
            .assume_offset(self.offset())
    }

    fn end_of_month(&self) -> Self {
        let days_in_month = match self.month() {
            Month::January => 31,
            Month::February => {
                if self.year() % 4 == 0 {
                    29
                } else {
                    28
                }
            }
            Month::March => 31,
            Month::April => 30,
            Month::May => 31,
            Month::June => 30,
            Month::July => 31,
            Month::August => 31,
            Month::September => 30,
            Month::October => 31,
            Month::November => 30,
            Month::December => 31,
        };

        let date = Date::from_calendar_date(self.year(), self.month(), days_in_month).unwrap();
        date.with_time(Time::from_hms(23, 59, 59).unwrap())
            .assume_offset(self.offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{PrimitiveDateTime, Weekday};

    fn create_test_datetime() -> OffsetDateTime {
        let offset = UtcOffset::from_hms(8, 0, 0).unwrap();
        OffsetDateTime::now_utc()
            .to_offset(offset)
            .replace_date_time(PrimitiveDateTime::new(
                Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                Time::from_hms(14, 30, 45).unwrap(),
            ))
    }

    #[test]
    fn test_start_of_day() {
        let dt = create_test_datetime();
        let start = dt.start_of_day();
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
    }

    #[test]
    fn test_end_of_day() {
        let dt = create_test_datetime();
        let end = dt.end_of_day();
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
    }

    #[test]
    fn test_start_of_week() {
        let dt = create_test_datetime(); // Friday
        let start = dt.start_of_week();
        assert_eq!(start.weekday(), Weekday::Monday);
    }

    #[test]
    fn test_end_of_week() {
        let dt = create_test_datetime(); // Friday
        let end = dt.end_of_week();
        assert_eq!(end.weekday(), Weekday::Sunday);
    }

    #[test]
    fn test_start_of_month() {
        let dt = create_test_datetime();
        let start = dt.start_of_month();
        assert_eq!(start.day(), 1);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
    }

    #[test]
    fn test_end_of_month() {
        let dt = create_test_datetime();
        let end = dt.end_of_month();
        assert_eq!(end.day(), 31); // March has 31 days
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
    }

    #[test]
    fn test_to_display_string_with_offset() {
        // Create a fixed time with UTC+8 offset
        let time_with_offset = OffsetDateTime::now_utc()
            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
            .replace_date_time(PrimitiveDateTime::new(
                Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                Time::from_hms(12, 0, 0).unwrap(),
            ));

        // Test UTC+8
        let str_utc8 = time_with_offset.to_display_string(8);
        assert_eq!(str_utc8, "2024-03-15 12:00:00+08:00");

        // Test UTC+0
        let str_utc = time_with_offset.to_display_string(0);
        assert_eq!(str_utc, "2024-03-15 04:00:00+00:00");

        // Test UTC-8
        let str_utc_minus8 = time_with_offset.to_display_string(-8);
        assert_eq!(str_utc_minus8, "2024-03-14 20:00:00-08:00");
    }

    #[test]
    fn test_to_chinese_string() {
        let time_with_offset = OffsetDateTime::now_utc()
            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
            .replace_date_time(PrimitiveDateTime::new(
                Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                Time::from_hms(12, 0, 0).unwrap(),
            ));

        let chinese_str = time_with_offset.to_chinese_string();
        assert_eq!(chinese_str, "2024年03月15日 12时00分00秒 +08:00");
    }

    #[test]
    fn test_replace_time_with_seconds() {
        let dt = create_test_datetime();

        // Test replacing with 10:20:30
        let new_dt = dt.replace_time_with_seconds(37230).unwrap();
        assert_eq!(new_dt.hour(), 10);
        assert_eq!(new_dt.minute(), 20);
        assert_eq!(new_dt.second(), 30);

        // Test replacing with 1:01:00
        let new_dt = dt.replace_time_with_seconds(3660).unwrap();
        assert_eq!(new_dt.hour(), 1);
        assert_eq!(new_dt.minute(), 1);
        assert_eq!(new_dt.second(), 0);

        // Test invalid seconds
        assert!(dt.replace_time_with_seconds(-1).is_err());
        assert!(dt.replace_time_with_seconds(24 * 3600).is_err());
    }

    #[test]
    fn test_align_time_to() {
        let dt = create_test_datetime();

        // Test alignment to 5 minutes
        let aligned = dt.align_time_to(300).unwrap(); // 5 minutes = 300 seconds
        assert_eq!(aligned.hour(), 14);
        assert_eq!(aligned.minute(), 30);
        assert_eq!(aligned.second(), 0);

        // Test alignment to 5 seconds
        let dt = dt.replace_time(Time::from_hms(14, 30, 3).unwrap());
        let aligned = dt.align_time_to(5).unwrap();
        assert_eq!(aligned.hour(), 14);
        assert_eq!(aligned.minute(), 30);
        assert_eq!(aligned.second(), 0);

        // Test alignment to 1 hour
        let aligned = dt.align_time_to(3600).unwrap();
        assert_eq!(aligned.hour(), 14);
        assert_eq!(aligned.minute(), 0);
        assert_eq!(aligned.second(), 0);

        // Test invalid unit
        assert!(dt.align_time_to(0).is_err());
        assert!(dt.align_time_to(24 * 3600).is_err());
    }

    #[test]
    fn test_next_day() {
        let dt = create_test_datetime();
        let next = dt.next_day();
        assert_eq!(next.day(), 16); // March 16
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_hour() {
        let dt = create_test_datetime();
        let next = dt.next_hour();
        assert_eq!(next.day(), 15);
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_minute() {
        let dt = create_test_datetime();
        let next = dt.next_minute();
        assert_eq!(next.day(), 15);
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 31);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_second() {
        let dt = create_test_datetime();
        let next = dt.next_second();
        assert_eq!(next.day(), 15);
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 46);
    }

    #[test]
    fn test_next_day_month_boundary() {
        let dt = OffsetDateTime::now_utc()
            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
            .replace_date_time(PrimitiveDateTime::new(
                Date::from_calendar_date(2024, time::Month::March, 31).unwrap(),
                Time::from_hms(14, 30, 45).unwrap(),
            ));

        let next = dt.next_day();
        assert_eq!(next.month(), time::Month::April);
        assert_eq!(next.day(), 1);
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_next_day_year_boundary() {
        let dt = OffsetDateTime::now_utc()
            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
            .replace_date_time(PrimitiveDateTime::new(
                Date::from_calendar_date(2024, time::Month::December, 31).unwrap(),
                Time::from_hms(14, 30, 45).unwrap(),
            ));

        let next = dt.next_day();
        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), time::Month::January);
        assert_eq!(next.day(), 1);
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 45);
    }

    #[test]
    fn test_to_hour_seconds() {
        let dt = create_test_datetime();
        assert_eq!(dt.to_hour_seconds(), 50400); // 14 * 3600

        let dt = dt.replace_time(Time::from_hms(0, 30, 45).unwrap());
        assert_eq!(dt.to_hour_seconds(), 0);

        let dt = dt.replace_time(Time::from_hms(23, 59, 59).unwrap());
        assert_eq!(dt.to_hour_seconds(), 82800); // 23 * 3600
    }

    #[test]
    fn test_to_minute_seconds() {
        let dt = create_test_datetime();
        assert_eq!(dt.to_minute_seconds(), 52200); // 14 * 3600 + 30 * 60

        let dt = dt.replace_time(Time::from_hms(0, 30, 45).unwrap());
        assert_eq!(dt.to_minute_seconds(), 1800); // 30 * 60

        let dt = dt.replace_time(Time::from_hms(23, 59, 59).unwrap());
        assert_eq!(dt.to_minute_seconds(), 86340); // 23 * 3600 + 59 * 60
    }
}

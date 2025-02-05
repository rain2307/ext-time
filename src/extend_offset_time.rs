use anyhow::Context;
use time::{
    format_description::{self},
    macros::format_description as fd,
    Date, Duration, OffsetDateTime, Time, UtcOffset,
    Month,
};

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
    fn from_milliseconds(timestamp: u64, offset_hours: i8) -> anyhow::Result<OffsetDateTime>;

    /// Parse timestamp in seconds with timezone offset (hours from UTC)
    fn from_seconds(timestamp: u64, offset_hours: i8) -> anyhow::Result<OffsetDateTime>;

    /// Parse datetime from date string, time string and milliseconds with timezone
    fn from_date_time(
        date: &str,
        time: &str,
        milli: u64,
        offset_hours: i8,
    ) -> anyhow::Result<OffsetDateTime>;

    /// Parse datetime from simple format string (YYYYMMDD_HHMM) with timezone
    fn from_simple(dt: &str, offset_hours: i8) -> anyhow::Result<OffsetDateTime>;

    /// Convert date format from YYYYMMDD to YYYY.MM.DD
    fn convert_to_dot_date(input: &str) -> anyhow::Result<String>;

    /// Get current time with specified timezone offset (hours from UTC)
    fn now_with_offset(offset_hours: i8) -> OffsetDateTime {
        OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(offset_hours, 0, 0).unwrap())
    }
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

    fn from_milliseconds(timestamp: u64, offset_hours: i8) -> anyhow::Result<OffsetDateTime> {
        let seconds = timestamp / 1000;
        let millis = timestamp % 1000;
        let offset = UtcOffset::from_hms(offset_hours, 0, 0).context("Invalid offset hours")?;
        
        Ok(OffsetDateTime::from_unix_timestamp(seconds as i64)
            .context("Invalid timestamp")?
            .replace_millisecond(millis as u16)
            .context("Invalid milliseconds")?
            .to_offset(offset))
    }

    fn from_seconds(timestamp: u64, offset_hours: i8) -> anyhow::Result<OffsetDateTime> {
        let offset = UtcOffset::from_hms(offset_hours, 0, 0).context("Invalid offset hours")?;
        Ok(OffsetDateTime::from_unix_timestamp(timestamp as i64)
            .context("Invalid timestamp")?
            .to_offset(offset))
    }

    fn from_date_time(
        date_str: &str,
        time_str: &str,
        milli: u64,
        offset_hours: i8,
    ) -> anyhow::Result<OffsetDateTime> {
        let format = fd!(
            "[year][month][day] [hour]:[minute]:[second].[subsecond digits:3] [offset_hour \
             sign:mandatory]:[offset_minute]:[offset_second]"
        );
        let dt = format!(
            "{} {}.{:03} {:+03}:00:00",
            date_str, time_str, milli, offset_hours
        );
        let parsed = OffsetDateTime::parse(&dt, &format)?;
        Ok(parsed)
    }

    fn from_simple(dt: &str, offset_hours: i8) -> anyhow::Result<OffsetDateTime> {
        let format = fd!("[year][month][day]_[hour][minute] [offset_hour sign:mandatory]");
        let dt = format!("{} {:+03}", dt, offset_hours);
        let parsed = OffsetDateTime::parse(&dt, &format)?;
        Ok(parsed)
    }

    fn convert_to_dot_date(input: &str) -> anyhow::Result<String> {
        let parse_format = fd!("[year][month][day]");
        let date = time::Date::parse(input, &parse_format)?;

        let output_format = fd!("[year].[month].[day]");
        let formatted_date = date.format(&output_format)?;
        Ok(formatted_date)
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
        let date = Date::from_calendar_date(
            self.year(),
            self.month(),
            1,
        ).unwrap();
        date.with_time(Time::from_hms(0, 0, 0).unwrap())
            .assume_offset(self.offset())
    }

    fn end_of_month(&self) -> Self {
        let days_in_month = match self.month() {
            Month::January => 31,
            Month::February => if self.year() % 4 == 0 { 29 } else { 28 },
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
        
        let date = Date::from_calendar_date(
            self.year(),
            self.month(),
            days_in_month,
        ).unwrap();
        date.with_time(Time::from_hms(23, 59, 59).unwrap())
            .assume_offset(self.offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Weekday, PrimitiveDateTime};

    fn create_test_datetime() -> OffsetDateTime {
        let offset = UtcOffset::from_hms(8, 0, 0).unwrap();
        OffsetDateTime::now_utc()
            .to_offset(offset)
            .replace_date_time(
                PrimitiveDateTime::new(
                    Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                    Time::from_hms(14, 30, 45).unwrap()
                )
            )
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
            .replace_date_time(
                PrimitiveDateTime::new(
                    Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                    Time::from_hms(12, 0, 0).unwrap()
                )
            );
        
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
            .replace_date_time(
                PrimitiveDateTime::new(
                    Date::from_calendar_date(2024, time::Month::March, 15).unwrap(),
                    Time::from_hms(12, 0, 0).unwrap()
                )
            );
        
        let chinese_str = time_with_offset.to_chinese_string();
        assert_eq!(chinese_str, "2024年03月15日 12时00分00秒 +08:00");
    }
}

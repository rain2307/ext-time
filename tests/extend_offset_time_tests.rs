use ext_time::ExtOffsetDateTime;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

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
fn test_align_to() {
    let dt = create_test_datetime();

    // Test alignment to 5 minutes
    let aligned = dt.align_to(300).unwrap(); // 5 minutes = 300 seconds
    assert_eq!(aligned.hour(), 14);
    assert_eq!(aligned.minute(), 30);
    assert_eq!(aligned.second(), 0);

    // Test alignment to 5 seconds
    let dt = dt.replace_time(Time::from_hms(14, 30, 3).unwrap());
    let aligned = dt.align_to(5).unwrap();
    assert_eq!(aligned.hour(), 14);
    assert_eq!(aligned.minute(), 30);
    assert_eq!(aligned.second(), 0);

    // Test alignment to 1 hour
    let aligned = dt.align_to(3600).unwrap();
    assert_eq!(aligned.hour(), 14);
    assert_eq!(aligned.minute(), 0);
    assert_eq!(aligned.second(), 0);

    // Test invalid unit
    assert!(dt.align_to(0).is_err());
    // 24 * 3600 is a valid alignment interval (24 hours)
    let aligned = dt.align_to(24 * 3600).unwrap();
    assert_eq!(aligned.hour(), 0);
    assert_eq!(aligned.minute(), 0);
    assert_eq!(aligned.second(), 0);
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

#[test]
fn test_duration_to_time_same_day() {
    // Test case: current time 14:30:45, target time 20:00:00 (same day)
    let dt = create_test_datetime();
    let duration = dt.duration_to_time(20, 0, 0);

    // Expected duration: 5 hours 29 minutes 15 seconds
    assert_eq!(duration.whole_hours(), 5);
    assert_eq!(duration.whole_minutes() % 60, 29);
    assert_eq!(duration.whole_seconds() % 60, 15);
}

#[test]
fn test_duration_to_time_cross_day() {
    // Test case: current time 23:30:45, target time 08:00:00 (next day)
    let dt = create_test_datetime().replace_time(Time::from_hms(23, 30, 45).unwrap());
    let duration = dt.duration_to_time(8, 0, 0);

    // Expected duration: 8 hours 29 minutes 15 seconds
    assert_eq!(duration.whole_hours(), 8);
    assert_eq!(duration.whole_minutes() % 60, 29);
    assert_eq!(duration.whole_seconds() % 60, 15);
}

#[test]
fn test_duration_to_time_same_time() {
    // Test case: current time 14:30:45, target time 14:30:45 (same time)
    let dt = create_test_datetime();
    let duration = dt.duration_to_time(14, 30, 45);

    // Expected duration: 0 seconds
    assert_eq!(duration.whole_seconds(), 0);
}

#[test]
fn test_duration_to_time_just_before_midnight() {
    // Test case: current time 23:59:59, target time 00:00:00 (next day)
    let dt = create_test_datetime().replace_time(Time::from_hms(23, 59, 59).unwrap());
    let duration = dt.duration_to_time(0, 0, 0);

    // Expected duration: 1 second
    assert_eq!(duration.whole_seconds(), 1);
}

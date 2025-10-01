use ext_time::ExtTime;
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
    let t = <time::Time as ExtTime>::from_str("9:30").unwrap();
    assert_eq!(t.hour(), 9);
    assert_eq!(t.minute(), 30);

    assert!(<time::Time as ExtTime>::from_str("25:00").is_err());
    assert!(<time::Time as ExtTime>::from_str("invalid").is_err());
}

#[test]
fn test_sub_ext() {
    let t1 = time!(23:00);
    let t2 = time!(1:00);
    assert_eq!(t1.sub_ext(t2), time::Duration::hours(22));
    assert_eq!(t2.sub_ext(t1), time::Duration::hours(2));
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
    let t = <time::Time as ExtTime>::from_seconds(37230).unwrap(); // 10:20:30
    assert_eq!(t.hour(), 10);
    assert_eq!(t.minute(), 20);
    assert_eq!(t.second(), 30);

    let t = <time::Time as ExtTime>::from_seconds(3660).unwrap(); // 1:01:00
    assert_eq!(t.hour(), 1);
    assert_eq!(t.minute(), 1);
    assert_eq!(t.second(), 0);

    assert!(<time::Time as ExtTime>::from_seconds(-1).is_err());
    assert!(<time::Time as ExtTime>::from_seconds(24 * 3600).is_err());
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
    // 24 * 3600 is a valid alignment interval (24 hours)
    let aligned = t.align_to(24 * 3600).unwrap();
    assert_eq!(aligned.hour(), 0);
    assert_eq!(aligned.minute(), 0);
    assert_eq!(aligned.second(), 0);
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
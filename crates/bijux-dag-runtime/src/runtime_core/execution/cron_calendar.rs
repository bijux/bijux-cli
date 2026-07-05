use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

pub(crate) fn validate_cron_expression(expression: &str) -> Result<(), String> {
    parse_cron_schedule(expression).map(|_| ())
}

pub(crate) fn validate_cron_timezone(timezone: &str) -> Result<(), String> {
    parse_cron_timezone(timezone).map(|_| ())
}

pub(crate) fn cron_matches_unix_ms(
    expression: &str,
    timezone: &str,
    unix_ms: u128,
) -> Result<bool, String> {
    if unix_ms % 1_000 != 0 {
        return Ok(false);
    }
    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    let current = utc_datetime_from_unix_ms(unix_ms)?.with_timezone(&timezone);
    Ok(schedule.includes(current))
}

pub(crate) fn next_cron_fire_unix_ms(
    expression: &str,
    timezone: &str,
    unix_ms: u128,
) -> Result<Option<u128>, String> {
    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    let anchor =
        utc_datetime_from_unix_ms(unix_ms)?.with_timezone(&timezone) - Duration::milliseconds(1);
    schedule.after(&anchor).next().map(unix_ms_from_datetime).transpose()
}

pub(crate) fn materialize_next_cron_runs(
    expression: &str,
    timezone: &str,
    unix_ms: u128,
    limit: usize,
) -> Result<Vec<u128>, String> {
    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    let anchor =
        utc_datetime_from_unix_ms(unix_ms)?.with_timezone(&timezone) - Duration::milliseconds(1);
    schedule.after(&anchor).take(limit.max(1)).map(unix_ms_from_datetime).collect()
}

pub(crate) fn cron_fire_times_between(
    expression: &str,
    timezone: &str,
    start_unix_ms: u128,
    end_unix_ms: u128,
    limit: usize,
) -> Result<Vec<u128>, String> {
    if start_unix_ms >= end_unix_ms || limit == 0 {
        return Ok(Vec::new());
    }

    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    let anchor = utc_datetime_from_unix_ms(start_unix_ms)?.with_timezone(&timezone)
        + Duration::milliseconds(1);

    let mut fire_times = Vec::new();
    for scheduled in schedule.after(&anchor) {
        let scheduled_unix_ms = unix_ms_from_datetime(scheduled)?;
        if scheduled_unix_ms > end_unix_ms {
            break;
        }
        fire_times.push(scheduled_unix_ms);
        if fire_times.len() >= limit {
            break;
        }
    }
    Ok(fire_times)
}

fn parse_cron_schedule(expression: &str) -> Result<Schedule, String> {
    let normalized = normalize_five_field_expression(expression)?;
    Schedule::from_str(&normalized).map_err(|error| format!("invalid cron expression: {error}"))
}

fn normalize_five_field_expression(expression: &str) -> Result<String, String> {
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return Err("cron expression must have exactly five fields".to_string());
    }
    Ok(format!("0 {}", fields.join(" ")))
}

fn parse_cron_timezone(timezone: &str) -> Result<Tz, String> {
    timezone.parse::<Tz>().map_err(|_| format!("unsupported cron timezone '{timezone}'"))
}

fn utc_datetime_from_unix_ms(unix_ms: u128) -> Result<DateTime<Utc>, String> {
    let unix_ms = i64::try_from(unix_ms)
        .map_err(|_| format!("unix timestamp exceeds supported range: {unix_ms}"))?;
    DateTime::<Utc>::from_timestamp_millis(unix_ms)
        .ok_or_else(|| format!("unix timestamp exceeds supported range: {unix_ms}"))
}

fn unix_ms_from_datetime<Z: TimeZone>(datetime: DateTime<Z>) -> Result<u128, String> {
    u128::try_from(datetime.timestamp_millis())
        .map_err(|_| "cron calendar produced a negative unix timestamp".to_string())
}

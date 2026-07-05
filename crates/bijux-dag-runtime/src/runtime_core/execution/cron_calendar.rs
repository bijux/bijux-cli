use chrono::{DateTime, LocalResult, TimeZone, Utc};
use chrono_tz::Tz;
use croner::Cron;
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
    schedule.is_time_matching(&current).map_err(|error| format!("invalid cron expression: {error}"))
}

pub(crate) fn next_cron_fire_unix_ms(
    expression: &str,
    timezone: &str,
    unix_ms: u128,
) -> Result<Option<u128>, String> {
    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    next_cron_occurrence(&schedule, timezone, unix_ms, true)
}

pub(crate) fn materialize_next_cron_runs(
    expression: &str,
    timezone: &str,
    unix_ms: u128,
    limit: usize,
) -> Result<Vec<u128>, String> {
    let schedule = parse_cron_schedule(expression)?;
    let timezone = parse_cron_timezone(timezone)?;
    collect_upcoming_cron_runs(&schedule, timezone, unix_ms, limit.max(1))
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
    let mut fire_times = Vec::new();
    let mut cursor_unix_ms = start_unix_ms;
    while fire_times.len() < limit {
        let Some(scheduled_unix_ms) =
            next_cron_occurrence(&schedule, timezone, cursor_unix_ms, false)?
        else {
            break;
        };
        if scheduled_unix_ms > end_unix_ms {
            break;
        }
        fire_times.push(scheduled_unix_ms);
        cursor_unix_ms = scheduled_unix_ms;
    }
    Ok(fire_times)
}

fn parse_cron_schedule(expression: &str) -> Result<Cron, String> {
    validate_five_field_expression(expression)?;
    Cron::from_str(expression).map_err(|error| format!("invalid cron expression: {error}"))
}

fn collect_upcoming_cron_runs(
    schedule: &Cron,
    timezone: Tz,
    start_unix_ms: u128,
    limit: usize,
) -> Result<Vec<u128>, String> {
    let mut runs = Vec::with_capacity(limit);
    let mut cursor_unix_ms = start_unix_ms;
    let mut include_current_exact_match = true;
    while runs.len() < limit {
        let Some(scheduled_unix_ms) =
            next_cron_occurrence(schedule, timezone, cursor_unix_ms, include_current_exact_match)?
        else {
            break;
        };
        runs.push(scheduled_unix_ms);
        cursor_unix_ms = scheduled_unix_ms;
        include_current_exact_match = false;
    }
    Ok(runs)
}

fn cron_search_limit_exceeded(error: &croner::errors::CronError) -> bool {
    matches!(error, croner::errors::CronError::TimeSearchLimitExceeded)
}

fn next_cron_occurrence(
    schedule: &Cron,
    timezone: Tz,
    unix_ms: u128,
    include_current_exact_match: bool,
) -> Result<Option<u128>, String> {
    let current = utc_datetime_from_unix_ms(unix_ms)?.with_timezone(&timezone);
    if include_current_exact_match
        && unix_ms % 1_000 == 0
        && schedule
            .is_time_matching(&current)
            .map_err(|error| format!("invalid cron expression: {error}"))?
    {
        return Ok(Some(unix_ms));
    }
    if !include_current_exact_match && unix_ms % 1_000 == 0 {
        if let Some(duplicate_unix_ms) = next_ambiguous_duplicate_unix_ms(schedule, &current)? {
            return Ok(Some(duplicate_unix_ms));
        }
    }

    let search_unix_ms = next_search_unix_ms(unix_ms);
    let search_from = utc_datetime_from_unix_ms(search_unix_ms)?.with_timezone(&timezone);
    match schedule.find_next_occurrence(&search_from, true) {
        Ok(next) => unix_ms_from_datetime(next).map(Some),
        Err(error) if cron_search_limit_exceeded(&error) => Ok(None),
        Err(error) => Err(format!("invalid cron expression: {error}")),
    }
}

fn next_ambiguous_duplicate_unix_ms(
    schedule: &Cron,
    current: &DateTime<Tz>,
) -> Result<Option<u128>, String> {
    let LocalResult::Ambiguous(first, second) =
        current.timezone().from_local_datetime(&current.naive_local())
    else {
        return Ok(None);
    };

    let (earlier, later) = if first.timestamp_millis() <= second.timestamp_millis() {
        (first, second)
    } else {
        (second, first)
    };
    if current.timestamp_millis() != earlier.timestamp_millis() {
        return Ok(None);
    }

    if schedule
        .is_time_matching(&later)
        .map_err(|error| format!("invalid cron expression: {error}"))?
    {
        return unix_ms_from_datetime(later).map(Some);
    }
    Ok(None)
}

fn validate_five_field_expression(expression: &str) -> Result<(), String> {
    let fields = expression.split_whitespace().count();
    if fields != 5 {
        return Err("cron expression must have exactly five fields".to_string());
    }
    Ok(())
}

fn next_search_unix_ms(unix_ms: u128) -> u128 {
    let remainder = unix_ms % 1_000;
    if remainder == 0 {
        unix_ms.saturating_add(1_000)
    } else {
        unix_ms + (1_000 - remainder)
    }
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

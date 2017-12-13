//! Convenient millisecond-clocks and delays.
//!
//! Heavily inspired on CZMQ's `zclock` class.
//!
//! ```
//! use neuras::clock::Clock;
//!
//! let clock = Clock::new();
//!
//! // Returns current system clock (non-monotonic) as milliseconds.
//! let start: i64 = clock.time().unwrap();
//!
//! // Sleep for a number or milliseconds.
//! let _ = clock.sleep(1_000);
//! let delta = clock.time().unwrap() - start;
//! assert!(delta >= 1_000);
//!
//!
//! // Returns current monotonic clock as milliseconds.
//! let start: i64 = clock.mono();
//!
//! // Sleep for a number or milliseconds.
//! let _ = clock.sleep(2_000);
//! let delta = clock.mono() - start;
//! assert!(delta >= 2_000);
//!
//!
//! // Returns current monotonic clock as microseconds.
//! let start: i64 = clock.usecs();
//!
//! // Sleep for a number or milliseconds.
//! let _ = clock.sleep(2);
//! let delta = clock.usecs() - start;
//! assert!(delta >= 2_000); //results can only be approximated at this resolution.
//!
//! // Return formatted RFC 3339 UTC date/time string.
//! let time_str: String = clock.time_str().unwrap();
//! ```
pub mod errors {
    //! Clock errors.
    error_chain! {
        errors {
            SysClockBeforeEpoch {
                description("clock time before UNIX EPOCH!")
            }
            ClockSystemDateTime {
                description("clock datetime string failed")
            }
            ClockSystemTime {
                description("clock system time failed")
            }
        }
    }
}

use self::errors::*;

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime, Utc};


// Convert `std::time::Duration` to microseconds.
fn duration_to_micros(d: Duration) -> i64 {
    let sec_to_micros = d.as_secs() as f64 * 1e6;
    let nanos_to_micros = d.subsec_nanos() as f64 * 1e-3;
    let duration = sec_to_micros + nanos_to_micros;
    duration as i64
}

// Convert `std::time::Duration` to milliseconds.
fn duration_to_millis(d: Duration) -> i64 {
    let sec_to_millis = d.as_secs() as f64 * 1e3;
    let nanos_to_millis = d.subsec_nanos() as f64 * 1e-6;
    let duration = sec_to_millis + nanos_to_millis;
    duration as i64
}

// Get the system time as the duration since UNIX EPOCH.
fn get_system_time() -> Result<Duration> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .chain_err(|| ErrorKind::SysClockBeforeEpoch)
}

/// A new `Clock` instance is created with the `std::time::Instant` it was created.
pub fn clock_new() -> Clock {
    Clock {
        start: Instant::now(),
    }
}

/// Sleep for a number of milliseconds.
pub fn clock_sleep(ms: u64) {
    ::std::thread::sleep(::std::time::Duration::from_millis(ms))
}

/// Returns monotonic clock in milliseconds.
pub fn clock_mono(clock: &Clock) -> i64 {
    let timestamp = clock.start.elapsed();
    duration_to_millis(timestamp)
}

/// Returns monotonic clock in microseconds.
pub fn clock_usecs(clock: &Clock) -> i64 {
    let timestamp = clock.start.elapsed();
    duration_to_micros(timestamp)
}

/// Returns monotonic clock in milliseconds.
pub fn clock_time() -> Result<i64> {
    let timestamp = get_system_time()
        .chain_err(|| ErrorKind::ClockSystemTime)?;
    let s = duration_to_millis(timestamp);
    Ok(s)
}

/// Returns an RFC 3339 and ISO 8601 UTC date and time string.
pub fn clock_time_str() -> Result<String> {
    let timestamp = get_system_time()
        .chain_err(|| ErrorKind::ClockSystemDateTime)?;
    let ndt = NaiveDateTime::from_timestamp(
        timestamp.as_secs() as i64,
        timestamp.subsec_nanos() as u32,
        );
    let dt = DateTime::<Utc>::from_utc(ndt, Utc);
    let dt_str = dt.to_rfc3339();
    Ok(dt_str)
}

/// Convenient API for millisecond clocks and delays.
#[derive(Copy, Clone, Debug)]
pub struct Clock {
    start: Instant,
}

impl Clock {
    /// A new `Clock` instance is created with the `std::time::Instant` it was created.
    pub fn new() -> Clock {
        clock_new()
    }

    /// Sleep for a number of milliseconds.
    pub fn sleep(&self, ms: u64) {
        clock_sleep(ms)
    }

    /// Returns monotonic clock in milliseconds.
    pub fn mono(&self) -> i64 {
        clock_mono(&self)
    }

    /// Returns monotonic clock in microseconds.
    pub fn usecs(&self) -> i64 {
        clock_usecs(&self)
    }

    /// Returns monotonic clock in milliseconds.
    pub fn time(&self) -> Result<i64> {
        clock_time()
    }

    /// Returns an RFC 3339 and ISO 8601 UTC date and time string.
    pub fn time_str(&self) -> Result<String> {
        clock_time_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use std::time::{Duration, Instant};

    #[test]
    fn clock_sleep_for_valid_msecs() {
        let start = Instant::now();
        let clock = Clock::new();
        let _ = clock.sleep(1_000);
        let duration = Instant::now() - start;
        assert_eq!(duration_to_millis(duration), 1_000);
    }

    #[test]
    fn clock_time_returns_milliseconds_from_unix_epoch() {
        let clock = Clock::new();
        let now = clock.time().unwrap() / 1_000;
        let dt = NaiveDateTime::from_timestamp(now as i64, 0);
        assert_eq!(dt.timestamp(), now);
    }

    #[test]
    fn clock_time_str_is_a_valid_rfc_3339_string() {
        let clock = Clock::new();
        let now = clock.time_str().unwrap();
        let dt = DateTime::parse_from_rfc3339(&now);
        assert!(dt.is_ok());
    }

    #[test]
    fn converts_duration_to_micros() {
        let dur = Duration::from_millis(1000);
        let micros = duration_to_micros(dur);
        assert_eq!(micros, 1_000_000);
    }

    #[test]
    fn converts_duration_to_millis() {
        let dur = Duration::from_millis(1000);
        let millis = duration_to_millis(dur);
        assert_eq!(millis, 1_000);
    }
}

pub use super::utils::Offset;
pub use super::traits::{TimeZone, TimeSystem};
use super::utils::Errors;
use super::instant::{Era, Instant};
use super::julian::SECONDS_PER_DAY;
use std::fmt;
use std::marker::Sized;

// There is no way to define a constant map in Rust (yet), so we're combining several structures
// to store when the leap seconds should be added. An updated list of leap seconds can be found
// here: https://www.ietf.org/timezones/data/leap-seconds.list .
const JANUARY_YEARS: [i32; 17] = [
    1972,
    1973,
    1974,
    1975,
    1976,
    1977,
    1978,
    1979,
    1980,
    1988,
    1990,
    1991,
    1996,
    1999,
    2006,
    2009,
    2017,
];

const JULY_YEARS: [i32; 11] = [
    1972,
    1981,
    1982,
    1983,
    1985,
    1992,
    1993,
    1994,
    1997,
    2012,
    2015,
];

pub const USUAL_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const USUAL_DAYS_PER_YEAR: f64 = 365.0;

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Utc is the interface between a time system and a time zone. All time zones are defined with
/// respect to UTC. Moreover, Utc inherently supports the past leap seconds, as reported by the
/// IETF and NIST at https://www.ietf.org/timezones/data/leap-seconds.list . NOTE: leap seconds
/// cannot be predicted! This module will be updated as soon as possible after a new leap second
/// has been announced.
/// **WARNING**: The historical oddities with calendars are not yet supported.
/// Moreover, despite the fields of Utc being public, it is strongly advised to use the `new`
/// function to ensure proper bound checking and correct leap second support. If your code breaks
/// because you're _not_ using `new`, don't file a bug.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Utc {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanos: u32,
}

impl TimeZone for Utc
where
    Self: Sized,
{
    /// Returns the offset between this TimeZone and UTC. In this case, the offset is strictly zero.
    fn utc_offset() -> Offset {
        Offset::new(0, 0, Era::Present)
    }

    /// Creates a new UTC date, with support for all the leap seconds with respect to TAI.
    /// *NOTE:* UTC leap seconds may be confusing because several dates have the **same** number
    /// of seconds since TAI epoch.
    /// **WARNING:** Does not support automatic carry and will return an error if so.
    /// **WARNING:** Although `PartialOrd` is implemented for Utc, the ambiguity of leap seconds
    /// as explained elsewhere in this documentation may lead to odd results (cf. examples below).
    ///
    /// # Examples
    /// ```
    /// use hifitime::utc::{Utc, TimeZone, TimeSystem};
    /// use hifitime::instant::{Duration, Era, Instant};
    /// use hifitime::julian::ModifiedJulian;
    ///
    /// let epoch = Utc::new(1900, 01, 01, 0, 0, 0, 0).expect("epoch failed");
    /// assert_eq!(
    ///     epoch.as_instant(),
    ///     Instant::new(0, 0, Era::Present),
    ///     "Incorrect Epoch computed"
    /// );
    ///
    /// assert_eq!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 leap second failed")
    ///         .as_instant(),
    ///     Instant::new(2272060799, 0, Era::Present),
    ///     "Incorrect January 1972 pre-leap second number computed"
    /// );
    /// assert_eq!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant(),
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant(),
    ///     "Incorrect January 1972 leap second number computed"
    /// );
    ///
    /// // Example of odd behavior when comparing/ordering dates using Utc or `as_instant`
    /// // Utc order claims (correctly) that the 60th second is _after_ the 59th. But the instant
    /// // is actually different because the 60th second is where we've inserted the leap second.
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Utc::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 59, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant() ==
    ///         Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .as_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// // Hence one second after the leap second, we get the following behavior (note the change
    /// // from equality to less when comparing via instant).
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0).expect(
    ///         "January 1972 1 second before leap second failed",
    ///     ) <
    ///         Utc::new(1972, 01, 01, 00, 00, 00, 0).expect(
    ///             "January 1972 1 second before leap second failed",
    ///         ),
    ///     "60th second should have a different instant than 59th second"
    /// );
    /// assert!(
    ///     Utc::new(1971, 12, 31, 23, 59, 60, 0)
    ///         .expect("January 1972 1 second before leap second failed")
    ///         .as_instant() <
    ///         Utc::new(1972, 01, 01, 00, 00, 00, 0)
    ///             .expect("January 1972 1 second before leap second failed")
    ///             .as_instant(),
    ///     "60th second should have a different instant than 59th second"
    /// );
    ///
    /// let santa = Utc::new(2017, 12, 25, 01, 02, 14, 0).expect("Xmas failed");
    ///
    /// assert_eq!(
    ///     santa.as_instant() + Duration::new(3600, 0),
    ///     Utc::new(2017, 12, 25, 02, 02, 14, 0)
    ///         .expect("Xmas failed")
    ///         .as_instant(),
    ///     "Could not add one hour to Christmas"
    /// );
    /// assert_eq!(format!("{}", santa), "2017-12-25T01:02:14+00:00");
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.as_instant()).days,
    ///     58112.043217592596
    /// );
    /// assert_eq!(
    ///     ModifiedJulian::from_instant(santa.as_instant()).julian_days(),
    ///     2458112.5432175924
    /// );
    /// ```
    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<Utc, Errors> {
        let mut max_seconds: u8 = 59;
        if (month == 12 || month == 6) && day == USUAL_DAYS_PER_MONTH[month as usize - 1] &&
            hour == 23 && minute == 59
        {
            if (month == 6 && JULY_YEARS.contains(&year)) ||
                (month == 12 && JANUARY_YEARS.contains(&(year + 1)))
            {
                max_seconds = 60;
            }
        }
        // General incorrect date times
        if month == 0 || month > 12 || day == 0 || day > 31 || hour > 24 || minute > 59 ||
            second > max_seconds || nanos as f64 > 1e9
        {
            return Err(Errors::Carry);
        }
        if day > USUAL_DAYS_PER_MONTH[month as usize - 1] {
            if month != 2 || !is_leap_year(year) {
                // Not in February or not a leap year
                return Err(Errors::Carry);
            }
        }
        Ok(Utc {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanos,
        })
    }
}

impl TimeSystem for Utc {
    /// `from_instant` converts an Instant to a Utc.
    /// Use this method to convert between different `TimeSystem` implementors.
    fn from_instant(instant: Instant) -> Utc {
        let (year, year_fraction) = quorem(instant.secs() as f64, 365.0 * SECONDS_PER_DAY);
        let (mut month, month_fraction) = quorem(year_fraction, 30.4365 * SECONDS_PER_DAY);
        month += 1; // Otherwise the month count starts at 0
        let mut days_this_month = USUAL_DAYS_PER_MONTH[(month - 1) as usize];
        if month == 2 && is_leap_year(year) {
            days_this_month += 1;
        }
        let (mut day, day_fraction) =
            quorem(month_fraction, SECONDS_PER_DAY * days_this_month as f64);
        day += 1; // Otherwise the day count starts at 0
        let (hours, hours_fraction) = quorem(day_fraction, 60.0 * 60.0);
        let (mins, secs) = quorem(hours_fraction, 60.0);
        match instant.era() {
            Era::Past => {
                Utc::new(
                    1900 - year,
                    month as u8,
                    day as u8,
                    hours as u8,
                    mins as u8,
                    secs as u8,
                    instant.nanos(),
                ).expect("date computed from instant is invalid (past)")
            }
            Era::Present => {
                Utc::new(
                    1900 + year,
                    month as u8,
                    day as u8,
                    hours as u8,
                    mins as u8,
                    secs as u8,
                    instant.nanos(),
                ).expect("date computed from instant is invalid")
            }
        }
    }

    /// `as_instant` returns an Instant from the Utc.
    /// Also use this method to convert between different `TimeSystem` implementors
    fn as_instant(self) -> Instant {
        let era: Era;
        if self.year >= 1900 {
            era = Era::Present;
        } else {
            era = Era::Past;
        }

        let mut seconds_wrt_1900: f64 = ((self.year - 1900).abs() as f64) * SECONDS_PER_DAY *
            USUAL_DAYS_PER_YEAR;

        // Now add the seconds for all the years prior to the current year
        for year in 1900..self.year {
            if is_leap_year(year) {
                seconds_wrt_1900 += SECONDS_PER_DAY;
            }
        }
        // Add the seconds for the months prior to the current month
        for month in 0..self.month - 1 {
            seconds_wrt_1900 += SECONDS_PER_DAY * USUAL_DAYS_PER_MONTH[(month) as usize] as f64;
        }
        if is_leap_year(self.year) && ((self.month == 2 && self.day == 29) || self.month > 2) {
            seconds_wrt_1900 += SECONDS_PER_DAY;
        }
        seconds_wrt_1900 += (self.day - 1) as f64 * SECONDS_PER_DAY + self.hour as f64 * 3600.0 +
            self.minute as f64 * 60.0 +
            self.second as f64;
        if self.second == 60 {
            // Herein lies the whole ambiguity of leap seconds. Two different UTC dates exist at the
            // same number of second afters J1900.0.
            seconds_wrt_1900 -= 1.0;
        }
        Instant::new(seconds_wrt_1900 as u64, self.nanos as u32, era)
    }
}

impl fmt::Display for Utc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )
    }
}

/// quorem returns a tuple of the quotient and the remainder a numerator and a denominator.
fn quorem(numerator: f64, denominator: f64) -> (i32, f64) {
    if numerator < 0.0 || denominator < 0.0 {
        panic!("quorem only supports positive numbers");
    }
    if denominator == 0.0 {
        panic!("cannot divide by zero");
    }
    (
        (numerator / denominator).floor() as i32,
        (numerator % denominator),
    )
}

#[test]
fn quorem_nominal_test() {
    assert_eq!(quorem(24.0, 6.0), (4, 0.0));
    assert_eq!(quorem(25.0, 6.0), (4, 1.0));
    assert_eq!(quorem(6.0, 6.0), (1, 0.0));
    assert_eq!(quorem(5.0, 6.0), (0, 5.0));
    assert_eq!(quorem(3540.0, 3600.0), (0, 3540.0));
    assert_eq!(quorem(3540.0, 60.0), (59, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_num_test() {
    assert_eq!(quorem(-24.0, 6.0), (4, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_den_test() {
    assert_eq!(quorem(24.0, -6.0), (4, 0.0));
}

#[test]
#[should_panic]
fn quorem_negative_numden_test() {
    // A valid argument could be made that this test should work, but there is no situation in
    // this library where two negative numbers should be considered a valid input.
    assert_eq!(quorem(-24.0, -6.0), (4, 0.0));
}

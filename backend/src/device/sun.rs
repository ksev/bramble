//! The `sun` crate is a library for calculating the position of the sun and sun phases
//! (like sunrise, sunset).
//! It is a port of the `JavaScript` library
//! [suncalc](https://github.com/mourner/suncalc).
//!
//! # Example
//!
//! ```
//! let unixtime = 1362441600000;
//! let lat = 48.0;
//! let lon = 9.0;
//! let pos = sun::pos(unixtime,lat,lon);
//! let az  = pos.azimuth.to_degrees();
//! let alt = pos.altitude.to_degrees();
//! println!("The position of the sun is {}/{}", az, alt);
//!
//! // calculate time of sunrise
//! let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
//! assert_eq!(time_ms, 1362463116241);
//! ```

use std::f64::consts::PI;

use time::OffsetDateTime;

// date/time constants and conversions

const MILLISECONDS_PER_DAY: u32 = 1000 * 60 * 60 * 24;
const J0: f64 = 0.0009;
const J1970: u32 = 2_440_588;
const J2000: u32 = 2_451_545;
const TO_RAD: f64 = PI / 180.0;
const OBLIQUITY_OF_EARTH: f64 = 23.4397 * TO_RAD;
const PERIHELION_OF_EARTH: f64 = 102.9372 * TO_RAD;

fn to_julian(unixtime_in_ms: i64) -> f64 {
    unixtime_in_ms as f64 / (MILLISECONDS_PER_DAY as f64) - 0.5 + J1970 as f64
}

fn from_julian(j: f64) -> i64 {
    ((j + 0.5 - J1970 as f64) * MILLISECONDS_PER_DAY as f64).round() as i64
}

fn to_days(unixtime_in_ms: i64) -> f64 {
    to_julian(unixtime_in_ms) - J2000 as f64
}

// general calculations for position
fn declination(l: f64, b: f64) -> f64 {
    (b.sin() * OBLIQUITY_OF_EARTH.cos() + b.cos() * OBLIQUITY_OF_EARTH.sin() * l.sin()).asin()
}

// general sun calculations

fn solar_mean_anomaly(d: f64) -> f64 {
    (357.5291 + 0.985_600_28 * d).to_radians()
}

fn equation_of_center(m: f64) -> f64 {
    (1.9148 * (1.0 * m).sin() + 0.02 * (2.0 * m).sin() + 0.0003 * (3.0 * m).sin()).to_radians()
}

fn ecliptic_longitude(m: f64) -> f64 {
    m + equation_of_center(m) + PERIHELION_OF_EARTH + PI
}

fn julian_cycle(d: f64, lw: f64) -> f64 {
    (d - J0 - lw / (2.0 * PI)).round()
}

fn approx_transit(ht: f64, lw: f64, n: f64) -> f64 {
    J0 + (ht + lw) / (2.0 * PI) + n
}

fn solar_transit_j(ds: f64, m: f64, l: f64) -> f64 {
    J2000 as f64 + ds + 0.0053 * m.sin() - 0.0069 * (2.0 * l).sin()
}

fn hour_angle(h: f64, phi: f64, d: f64) -> f64 {
    ((h.sin() - phi.sin() * d.sin()) / (phi.cos() * d.cos())).acos()
}

fn observer_angle(height: f64) -> f64 {
    -2.076 * height.sqrt() / 60.0
}

/// returns set time for the given sun altitude
fn get_set_j(h: f64, lw: f64, phi: f64, dec: f64, n: f64, m: f64, l: f64) -> f64 {
    let w = hour_angle(h, phi, dec);
    let a = approx_transit(w, lw, n);

    solar_transit_j(a, m, l)
}

/// Calculates the time for the given [`SunPhase`] at a given date, height and Latitude/Longitude.
/// The returned time is the [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
///
/// # Arguments
///
/// * `date`      - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `sun_phase` - [`SunPhase`] to calcuate time for
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
/// * `height`    - Observer height in meters above the horizon
///
/// # Examples
///
/// ```rust
/// // calculate time of sunrise
/// let unixtime = 1362441600000;
/// let lat = 48.0;
/// let lon = 9.0;
/// let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
/// assert_eq!(time_ms, 1362463116241);
/// ```

pub fn time_at_phase(
    date: OffsetDateTime,
    sun_phase: SunPhase,
    lat: f64,
    lon: f64,
    height: f64,
) -> OffsetDateTime {
    let lw = -lon.to_radians();
    let phi = lat.to_radians();

    let dh = observer_angle(height);
    let date = date.unix_timestamp() * 1000;

    let d = to_days(date);
    let n = julian_cycle(d, lw);
    let ds = approx_transit(0.0, lw, n);

    let m = solar_mean_anomaly(ds);
    let l = ecliptic_longitude(m);
    let dec = declination(l, 0.0);

    let j_noon = solar_transit_j(ds, m, l);

    let h0 = (sun_phase.angle_deg() + dh).to_radians();
    let j_set = get_set_j(h0, lw, phi, dec, n, m, l);

    let unix_ms = if sun_phase.is_rise() {
        let j_rise = j_noon - (j_set - j_noon);
        from_julian(j_rise)
    } else {
        from_julian(j_set)
    };

    OffsetDateTime::from_unix_timestamp(unix_ms / 1000).unwrap()
}

/// Sun phases for use with [`time_at_phase`].
#[derive(Clone, Copy, Debug)]
pub enum SunPhase {
    Sunrise,
    Sunset,
    SunriseEnd,
    SunsetStart,
}

impl SunPhase {
    fn angle_deg(&self) -> f64 {
        match self {
            SunPhase::Sunrise | SunPhase::Sunset => -0.833,
            SunPhase::SunriseEnd | SunPhase::SunsetStart => -0.5,
        }
    }

    fn is_rise(&self) -> bool {
        match self {
            SunPhase::Sunrise | SunPhase::SunriseEnd => true,
            SunPhase::Sunset | SunPhase::SunsetStart => false,
        }
    }
}

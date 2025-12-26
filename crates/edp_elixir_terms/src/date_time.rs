// Copyright (C) 2025 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Elixir Date, Time, and DateTime type support.

use erltf::{Atom, OwnedTerm};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents an Elixir Date (`~D[2025-12-25]`).
///
/// # Example
///
/// ```
/// use edp_elixir_terms::ElixirDate;
/// use erltf::OwnedTerm;
///
/// let date = ElixirDate::new(2025, 12, 25);
/// let term: OwnedTerm = date.into();
///
/// assert!(term.is_elixir_struct());
/// assert_eq!(term.elixir_struct_module(), Some("Elixir.Date"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElixirDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl ElixirDate {
    /// Creates a new Date without validation. Use `try_new` for validation.
    #[must_use]
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Creates a new Date with validation.
    #[must_use]
    pub fn try_new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }

        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => return None,
        };

        if day < 1 || day > max_day {
            return None;
        }

        Some(Self { year, month, day })
    }

    /// Returns true if the given year is a leap year.
    #[must_use]
    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Parses an OwnedTerm as a Date struct.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.Date") {
            return None;
        }

        let map = term.as_map()?;
        let year = map.get(&OwnedTerm::Atom(Atom::new("year")))?.as_integer()? as i32;
        let month = map
            .get(&OwnedTerm::Atom(Atom::new("month")))?
            .as_integer()? as u8;
        let day = map.get(&OwnedTerm::Atom(Atom::new("day")))?.as_integer()? as u8;

        Some(Self { year, month, day })
    }
}

impl From<ElixirDate> for OwnedTerm {
    fn from(date: ElixirDate) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.Date")),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("year")),
            OwnedTerm::Integer(date.year as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("month")),
            OwnedTerm::Integer(date.month as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("day")),
            OwnedTerm::Integer(date.day as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("calendar")),
            OwnedTerm::Atom(Atom::new("Elixir.Calendar.ISO")),
        );
        OwnedTerm::Map(map)
    }
}

impl std::fmt::Display for ElixirDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "~D[{:04}-{:02}-{:02}]", self.year, self.month, self.day)
    }
}

/// Represents an Elixir Time (`~T[14:30:00.000000]`).
///
/// # Example
///
/// ```
/// use edp_elixir_terms::ElixirTime;
/// use erltf::OwnedTerm;
///
/// let time = ElixirTime::new(14, 30, 0, 0, 0);
/// let term: OwnedTerm = time.into();
///
/// assert!(term.is_elixir_struct());
/// assert_eq!(term.elixir_struct_module(), Some("Elixir.Time"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElixirTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Microsecond value (0-999999)
    pub microsecond_value: u32,
    /// Precision (0-6)
    pub microsecond_precision: u8,
}

impl ElixirTime {
    /// Creates a new Time without validation. Use `try_new` for validation.
    #[must_use]
    pub fn new(hour: u8, minute: u8, second: u8, microsecond: u32, precision: u8) -> Self {
        Self {
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision.min(6),
        }
    }

    /// Creates a new Time with validation.
    #[must_use]
    pub fn try_new(
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
    ) -> Option<Self> {
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        if microsecond > 999_999 {
            return None;
        }
        if precision > 6 {
            return None;
        }
        Some(Self {
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision,
        })
    }

    /// Creates a Time without microseconds.
    #[must_use]
    pub fn hms(hour: u8, minute: u8, second: u8) -> Self {
        Self::new(hour, minute, second, 0, 0)
    }

    /// Creates a Time without microseconds, with validation.
    #[must_use]
    pub fn try_hms(hour: u8, minute: u8, second: u8) -> Option<Self> {
        Self::try_new(hour, minute, second, 0, 0)
    }

    /// Parses an OwnedTerm as a Time struct.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.Time") {
            return None;
        }

        let map = term.as_map()?;
        let hour = map.get(&OwnedTerm::Atom(Atom::new("hour")))?.as_integer()? as u8;
        let minute = map
            .get(&OwnedTerm::Atom(Atom::new("minute")))?
            .as_integer()? as u8;
        let second = map
            .get(&OwnedTerm::Atom(Atom::new("second")))?
            .as_integer()? as u8;

        let (microsecond_value, microsecond_precision) =
            if let Some(us) = map.get(&OwnedTerm::Atom(Atom::new("microsecond"))) {
                if let Some((val, prec)) = us.as_2_tuple() {
                    (val.as_integer()? as u32, prec.as_integer()? as u8)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        Some(Self {
            hour,
            minute,
            second,
            microsecond_value,
            microsecond_precision,
        })
    }
}

impl From<ElixirTime> for OwnedTerm {
    fn from(time: ElixirTime) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.Time")),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("hour")),
            OwnedTerm::Integer(time.hour as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("minute")),
            OwnedTerm::Integer(time.minute as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("second")),
            OwnedTerm::Integer(time.second as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("microsecond")),
            OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(time.microsecond_value as i64),
                OwnedTerm::Integer(time.microsecond_precision as i64),
            ]),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("calendar")),
            OwnedTerm::Atom(Atom::new("Elixir.Calendar.ISO")),
        );
        OwnedTerm::Map(map)
    }
}

impl std::fmt::Display for ElixirTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.microsecond_precision > 0 {
            let divisor = 10u32.pow(6 - self.microsecond_precision as u32);
            let frac = self.microsecond_value / divisor;
            write!(
                f,
                "~T[{:02}:{:02}:{:02}.{:0width$}]",
                self.hour,
                self.minute,
                self.second,
                frac,
                width = self.microsecond_precision as usize
            )
        } else {
            write!(
                f,
                "~T[{:02}:{:02}:{:02}]",
                self.hour, self.minute, self.second
            )
        }
    }
}

/// Represents an Elixir NaiveDateTime (`~N[2025-12-25 14:30:00]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElixirNaiveDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub microsecond_value: u32,
    pub microsecond_precision: u8,
}

impl ElixirNaiveDateTime {
    /// Creates a new NaiveDateTime without validation. Use `try_new` for validation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision.min(6),
        }
    }

    /// Creates a new NaiveDateTime with validation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
    ) -> Option<Self> {
        ElixirDate::try_new(year, month, day)?;
        ElixirTime::try_new(hour, minute, second, microsecond, precision)?;
        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision,
        })
    }

    /// Creates a NaiveDateTime from date and time components.
    #[must_use]
    pub fn from_date_time(date: ElixirDate, time: ElixirTime) -> Self {
        Self {
            year: date.year,
            month: date.month,
            day: date.day,
            hour: time.hour,
            minute: time.minute,
            second: time.second,
            microsecond_value: time.microsecond_value,
            microsecond_precision: time.microsecond_precision,
        }
    }

    /// Extracts the date component.
    #[must_use]
    pub fn to_date(&self) -> ElixirDate {
        ElixirDate::new(self.year, self.month, self.day)
    }

    /// Extracts the time component.
    #[must_use]
    pub fn to_time(&self) -> ElixirTime {
        ElixirTime::new(
            self.hour,
            self.minute,
            self.second,
            self.microsecond_value,
            self.microsecond_precision,
        )
    }

    /// Parses an OwnedTerm as a NaiveDateTime struct.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.NaiveDateTime") {
            return None;
        }

        let map = term.as_map()?;
        let year = map.get(&OwnedTerm::Atom(Atom::new("year")))?.as_integer()? as i32;
        let month = map
            .get(&OwnedTerm::Atom(Atom::new("month")))?
            .as_integer()? as u8;
        let day = map.get(&OwnedTerm::Atom(Atom::new("day")))?.as_integer()? as u8;
        let hour = map.get(&OwnedTerm::Atom(Atom::new("hour")))?.as_integer()? as u8;
        let minute = map
            .get(&OwnedTerm::Atom(Atom::new("minute")))?
            .as_integer()? as u8;
        let second = map
            .get(&OwnedTerm::Atom(Atom::new("second")))?
            .as_integer()? as u8;

        let (microsecond_value, microsecond_precision) =
            if let Some(us) = map.get(&OwnedTerm::Atom(Atom::new("microsecond"))) {
                if let Some((val, prec)) = us.as_2_tuple() {
                    (val.as_integer()? as u32, prec.as_integer()? as u8)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value,
            microsecond_precision,
        })
    }
}

impl std::fmt::Display for ElixirNaiveDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.microsecond_precision > 0 {
            let divisor = 10u32.pow(6 - self.microsecond_precision as u32);
            let frac = self.microsecond_value / divisor;
            write!(
                f,
                "~N[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:0width$}]",
                self.year,
                self.month,
                self.day,
                self.hour,
                self.minute,
                self.second,
                frac,
                width = self.microsecond_precision as usize
            )
        } else {
            write!(
                f,
                "~N[{:04}-{:02}-{:02} {:02}:{:02}:{:02}]",
                self.year, self.month, self.day, self.hour, self.minute, self.second
            )
        }
    }
}

impl From<ElixirNaiveDateTime> for OwnedTerm {
    fn from(dt: ElixirNaiveDateTime) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.NaiveDateTime")),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("year")),
            OwnedTerm::Integer(dt.year as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("month")),
            OwnedTerm::Integer(dt.month as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("day")),
            OwnedTerm::Integer(dt.day as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("hour")),
            OwnedTerm::Integer(dt.hour as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("minute")),
            OwnedTerm::Integer(dt.minute as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("second")),
            OwnedTerm::Integer(dt.second as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("microsecond")),
            OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(dt.microsecond_value as i64),
                OwnedTerm::Integer(dt.microsecond_precision as i64),
            ]),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("calendar")),
            OwnedTerm::Atom(Atom::new("Elixir.Calendar.ISO")),
        );
        OwnedTerm::Map(map)
    }
}

/// Represents an Elixir DateTime with timezone (`~U[2025-12-25 14:30:00Z]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElixirDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub microsecond_value: u32,
    pub microsecond_precision: u8,
    pub time_zone: String,
    pub zone_abbr: String,
    pub utc_offset: i32,
    pub std_offset: i32,
}

impl ElixirDateTime {
    /// Creates a UTC DateTime without validation. Use `try_utc` for validation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision.min(6),
            time_zone: "Etc/UTC".to_string(),
            zone_abbr: "UTC".to_string(),
            utc_offset: 0,
            std_offset: 0,
        }
    }

    /// Creates a UTC DateTime with validation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn try_utc(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
    ) -> Option<Self> {
        ElixirDate::try_new(year, month, day)?;
        ElixirTime::try_new(hour, minute, second, microsecond, precision)?;
        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision,
            time_zone: "Etc/UTC".to_string(),
            zone_abbr: "UTC".to_string(),
            utc_offset: 0,
            std_offset: 0,
        })
    }

    /// Creates a DateTime with full timezone information without validation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn with_timezone(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
        precision: u8,
        time_zone: &str,
        zone_abbr: &str,
        utc_offset: i32,
        std_offset: i32,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value: microsecond,
            microsecond_precision: precision.min(6),
            time_zone: time_zone.to_string(),
            zone_abbr: zone_abbr.to_string(),
            utc_offset,
            std_offset,
        }
    }

    /// Extracts the date component.
    #[must_use]
    pub fn to_date(&self) -> ElixirDate {
        ElixirDate::new(self.year, self.month, self.day)
    }

    /// Extracts the time component.
    #[must_use]
    pub fn to_time(&self) -> ElixirTime {
        ElixirTime::new(
            self.hour,
            self.minute,
            self.second,
            self.microsecond_value,
            self.microsecond_precision,
        )
    }

    /// Converts to a NaiveDateTime (discarding timezone).
    #[must_use]
    pub fn to_naive(&self) -> ElixirNaiveDateTime {
        ElixirNaiveDateTime::new(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.microsecond_value,
            self.microsecond_precision,
        )
    }

    /// Parses an OwnedTerm as a DateTime struct.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.DateTime") {
            return None;
        }

        let map = term.as_map()?;
        let year = map.get(&OwnedTerm::Atom(Atom::new("year")))?.as_integer()? as i32;
        let month = map
            .get(&OwnedTerm::Atom(Atom::new("month")))?
            .as_integer()? as u8;
        let day = map.get(&OwnedTerm::Atom(Atom::new("day")))?.as_integer()? as u8;
        let hour = map.get(&OwnedTerm::Atom(Atom::new("hour")))?.as_integer()? as u8;
        let minute = map
            .get(&OwnedTerm::Atom(Atom::new("minute")))?
            .as_integer()? as u8;
        let second = map
            .get(&OwnedTerm::Atom(Atom::new("second")))?
            .as_integer()? as u8;

        let (microsecond_value, microsecond_precision) =
            if let Some(us) = map.get(&OwnedTerm::Atom(Atom::new("microsecond"))) {
                if let Some((val, prec)) = us.as_2_tuple() {
                    (val.as_integer()? as u32, prec.as_integer()? as u8)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        let time_zone = map
            .get(&OwnedTerm::Atom(Atom::new("time_zone")))?
            .as_erlang_string()?;
        let zone_abbr = map
            .get(&OwnedTerm::Atom(Atom::new("zone_abbr")))?
            .as_erlang_string()?;
        let utc_offset = map
            .get(&OwnedTerm::Atom(Atom::new("utc_offset")))?
            .as_integer()? as i32;
        let std_offset = map
            .get(&OwnedTerm::Atom(Atom::new("std_offset")))?
            .as_integer()? as i32;

        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond_value,
            microsecond_precision,
            time_zone,
            zone_abbr,
            utc_offset,
            std_offset,
        })
    }
}

impl std::fmt::Display for ElixirDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zone = if self.zone_abbr == "UTC" {
            "Z"
        } else {
            &self.zone_abbr
        };
        if self.microsecond_precision > 0 {
            let divisor = 10u32.pow(6 - self.microsecond_precision as u32);
            let frac = self.microsecond_value / divisor;
            write!(
                f,
                "~U[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:0width$}{}]",
                self.year,
                self.month,
                self.day,
                self.hour,
                self.minute,
                self.second,
                frac,
                zone,
                width = self.microsecond_precision as usize
            )
        } else {
            write!(
                f,
                "~U[{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}]",
                self.year, self.month, self.day, self.hour, self.minute, self.second, zone
            )
        }
    }
}

impl From<ElixirDateTime> for OwnedTerm {
    fn from(dt: ElixirDateTime) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.DateTime")),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("year")),
            OwnedTerm::Integer(dt.year as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("month")),
            OwnedTerm::Integer(dt.month as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("day")),
            OwnedTerm::Integer(dt.day as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("hour")),
            OwnedTerm::Integer(dt.hour as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("minute")),
            OwnedTerm::Integer(dt.minute as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("second")),
            OwnedTerm::Integer(dt.second as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("microsecond")),
            OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(dt.microsecond_value as i64),
                OwnedTerm::Integer(dt.microsecond_precision as i64),
            ]),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("time_zone")),
            OwnedTerm::Binary(dt.time_zone.into_bytes()),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("zone_abbr")),
            OwnedTerm::Binary(dt.zone_abbr.into_bytes()),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("utc_offset")),
            OwnedTerm::Integer(dt.utc_offset as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("std_offset")),
            OwnedTerm::Integer(dt.std_offset as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("calendar")),
            OwnedTerm::Atom(Atom::new("Elixir.Calendar.ISO")),
        );
        OwnedTerm::Map(map)
    }
}

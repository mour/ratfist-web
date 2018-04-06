use std::collections::HashSet;
use regex::Regex;

use chrono::prelude::*;

use rocket::request::FromParam;
use rocket::request::FromFormValue;
use rocket::http::RawStr;

use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct IdRange(HashSet<u32>);

impl<'req> FromParam<'req> for IdRange {
    type Error = ();

    fn from_param(param: &'req RawStr) -> Result<Self, Self::Error> {

        let term_regex = Regex::new("(?P<from>[0-9]+)(?::(?P<to>[0-9]+))?").expect("invalid regex");

        let mut range = IdRange(HashSet::new());

        for term in param.split(',') {
            let caps = term_regex.captures(term).ok_or(())?;

            let from_str = caps.name("from").ok_or(())?.as_str();
            let from = from_str.parse().map_err(|_| ())?;

            if let Some(to_match) = caps.name("to") {
                let to = to_match.as_str().parse().map_err(|_| ())?;

                let (from, to) = if from <= to {
                    (from, to)
                } else {
                    (to, from)
                };

                for i in from..=to {
                    range.0.insert(i);
                }

            } else {
                range.0.insert(from);
            }
        }

        Ok(range)
    }
}

impl IntoIterator for IdRange {
    type Item = u32;
    type IntoIter = ::std::collections::hash_set::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}



#[derive(Debug, Serialize)]
pub struct DateTimeUtc(DateTime<Utc>);

impl Deref for DateTimeUtc {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromForm, Debug)]
pub struct TimeRangeExplicitTimes {
    pub from: DateTimeUtc,
    pub to: DateTimeUtc
}

#[derive(FromForm, Debug)]
pub struct TimeRangeOptionalEndTime {
    pub from: DateTimeUtc,
    pub to: Option<DateTimeUtc>
}

impl<'v> FromFormValue<'v> for DateTimeUtc {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<DateTimeUtc, &'v RawStr> {
        Ok(DateTimeUtc(form_value.parse::<DateTime<Utc>>().map_err(|_| form_value)?))
    }
}

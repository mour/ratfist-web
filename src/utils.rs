use std::collections::HashSet;
use regex::Regex;

use rocket::request::FromParam;
use rocket::http::RawStr;

#[derive(Debug)]
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

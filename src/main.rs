use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use ical::parser::ical::component::{IcalCalendar, IcalEvent};
use ical::IcalParser;
use reqwest::blocking::get;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Event {
    title: Option<String>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    location: Option<String>,
    description: Option<String>,
}

fn parse_ical_utc_date_time(value: Option<String>) -> Option<DateTime<Utc>> {
    let result = NaiveDateTime::parse_from_str(value?.as_str(), "%Y%m%dT%H%M%S");

    Some(result.ok()?.and_utc())
}

impl TryFrom<IcalEvent> for Event {
    type Error = anyhow::Error;
    fn try_from(value: IcalEvent) -> Result<Self> {
        let mut result = Event::default();

        for property in value.properties {
            match property.name.as_str() {
                "SUMMARY" => result.title = property.value,
                "DTSTART" => result.start = parse_ical_utc_date_time(property.value),
                "DTEND" => result.end = parse_ical_utc_date_time(property.value),
                "LOCATION" => result.location = property.value,
                "DESCRIPTION" => result.description = property.value,
                _ => (),
            }
        }
        Ok(result)
    }
}

fn main() -> Result<()> {
    let body = get("https://metalab.at/calendar/export/ical/")?.text()?;

    let reader = BufReader::new(body.as_bytes());
    let parser = IcalParser::new(reader);

    let calendar: IcalCalendar = parser
        .into_iter()
        .next()
        .context("no calendar found in ical file")?
        .context("")?;

    let events: Result<Vec<Event>> = calendar
        .events
        .into_iter()
        .map(|it| it.try_into())
        .collect();

    let events = events.context("could not convert events")?;

    let json = serde_json::to_string_pretty(&events)?;
    println!("{json}");

    Ok(())
}

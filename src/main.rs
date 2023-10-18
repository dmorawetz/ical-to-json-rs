use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use ical::parser::ical::component::IcalCalendar;
use reqwest::blocking::get;

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

impl TryFrom<ical::parser::ical::component::IcalEvent> for Event {
    type Error = anyhow::Error;
    fn try_from(value: ical::parser::ical::component::IcalEvent) -> Result<Self> {
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

    let reader = ical::IcalParser::new(std::io::BufReader::new(body.as_bytes()));

    let calendar: IcalCalendar = reader
        .into_iter()
        .next()
        .context("no calendar found in ical file")?
        .context("")?;

    let mut events: Vec<Event> = vec![];

    for event in calendar.events {
        events.push(event.try_into()?);
    }

    let json = serde_json::to_string(&events)?;
    println!("{json}");

    Ok(())
}

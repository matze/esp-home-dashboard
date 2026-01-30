use embedded_io_async::Read;

const MAX_SUMMARY_LENGTH: usize = 32;

pub enum Error {
    ParseEvent,
    DateTime,
    PushStr,
}

/// Sorts events by start date in ascending order.
pub fn sort_by_date(events: &mut [Event]) {
    events.sort_unstable_by(|a, b| a.start.cmp(&b.start));
}

#[derive(Default)]
pub struct Event {
    pub start: jiff::Zoned,
    pub end: jiff::Zoned,
    pub summary: heapless::String<MAX_SUMMARY_LENGTH>,
}

pub async fn parse<R>(
    mut reader: R,
    now: jiff::Zoned,
    events: &mut [Event],
) -> Result<&mut [Event], Error>
where
    R: Read,
{
    enum State {
        ScanVEvent,
        InVEvent,
    }

    let mut current_event = Event::default();
    let mut events_filled = 0;
    let mut state = State::ScanVEvent;
    let max_events = events.len();
    let timezone = now.time_zone().clone();

    loop {
        let mut line = [0u8; 160];

        match read_line(&mut reader, &mut line).await {
            Ok(size) => {
                let s = core::str::from_utf8(&line[..size]).map_err(|_| Error::ParseEvent)?;

                if s.starts_with("BEGIN:VEVENT") {
                    state = State::InVEvent;
                    continue;
                }

                if s.starts_with("END:VEVENT") {
                    if matches!(state, State::InVEvent) {
                        events[events_filled] = current_event;
                        events_filled += 1;
                    }

                    current_event = Event::default();
                    state = State::ScanVEvent;

                    if events_filled == max_events {
                        return Ok(&mut events[..max_events]);
                    }

                    continue;
                }

                if matches!(state, State::InVEvent) {
                    if s.starts_with("DTEND") {
                        let colon = s.find(':').expect("colon");

                        let Ok(end) = parse_ics_timestamp(&s[colon + 1..], timezone.clone()) else {
                            state = State::ScanVEvent;
                            continue;
                        };

                        if end < now {
                            state = State::ScanVEvent;
                        } else {
                            current_event.end = end;
                        }

                        continue;
                    }

                    if s.starts_with("DTSTART") {
                        let colon = s.find(':').expect("colon");

                        let Ok(start) = parse_ics_timestamp(&s[colon + 1..], timezone.clone())
                        else {
                            state = State::ScanVEvent;
                            continue;
                        };

                        current_event.start = start;
                        continue;
                    }

                    if let Some(summary) = s.strip_prefix("SUMMARY:") {
                        let summary = &summary[..MAX_SUMMARY_LENGTH.min(summary.len())];

                        current_event
                            .summary
                            .push_str(summary)
                            .map_err(|_| Error::PushStr)?;
                    }
                }
            }
            Err(ReadLineError::End) => {
                return Ok(&mut events[..events_filled]);
            }
            Err(ReadLineError::BufferFull) => {
                continue;
            }
        }
    }
}

fn parse_ics_timestamp(s: &str, timezone: jiff::tz::TimeZone) -> Result<jiff::Zoned, Error> {
    // TODO: also parse full day events without a time
    let datetime = jiff::civil::DateTime::strptime("%Y%m%dT%H%M%S", s.trim_end_matches('Z'))
        .map_err(|_| Error::DateTime)?;

    // If timestamp ends with 'Z', it's UTC - convert to local timezone
    // Otherwise, interpret as already in local timezone
    if s.ends_with('Z') {
        Ok(datetime
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map_err(|_| Error::DateTime)?
            .with_time_zone(timezone))
    } else {
        Ok(datetime.to_zoned(timezone).expect("making Zoned"))
    }
}

enum ReadLineError {
    BufferFull,
    End,
}

/// Reads a single line  into the provided buffer. Returns the number of bytes read (excluding `\r`
/// and `\n`).
async fn read_line<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize, ReadLineError> {
    let mut pos = 0;

    loop {
        if pos >= buf.len() {
            return Err(ReadLineError::BufferFull);
        }

        let mut byte = [0u8; 1];

        reader
            .read_exact(&mut byte)
            .await
            .map_err(|_| ReadLineError::End)?;

        match byte[0] {
            b'\n' => {
                return Ok(pos);
            }
            b'\r' => {
                // Ignore CR, wait for LF
                continue;
            }
            other => {
                buf[pos] = other;
                pos += 1;
            }
        }
    }
}

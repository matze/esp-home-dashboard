include!(concat!(env!("OUT_DIR"), "/font_spleen_8_16.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_12_24.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_16_32.rs"));

use core::convert::Infallible;

use embedded_graphics::image::Image;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::prelude::{Drawable, Point, Primitive, Size};
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder};
use epd_waveshare::epd7in5_v2::Display7in5;
use epd_waveshare::prelude::*;
use esp_backtrace as _;
use heapless::{String, format};
use jiff::civil::{Date, Weekday};
use jiff::fmt::strtime;

use crate::ics::Either;
use crate::{ics, weather};

const SPLEEN_HUGE_STYLE: MonoTextStyle<Color> = MonoTextStyleBuilder::new()
    .font(&FONT_SPLEEN_16_32)
    .text_color(Color::White)
    .background_color(Color::Black)
    .build();

const SPLEEN_LARGE_STYLE: MonoTextStyle<Color> = MonoTextStyleBuilder::new()
    .font(&FONT_SPLEEN_12_24)
    .text_color(Color::White)
    .background_color(Color::Black)
    .build();

const SPLEEN_SMALL_STYLE: MonoTextStyle<Color> = MonoTextStyleBuilder::new()
    .font(&FONT_SPLEEN_8_16)
    .text_color(Color::White)
    .background_color(Color::Black)
    .build();

const LINE_STYLE: PrimitiveStyle<Color> = PrimitiveStyle::with_stroke(Color::White, 1);

const TOP_TEXT_STYLE: TextStyle = TextStyleBuilder::new().baseline(Baseline::Top).build();

const BOTTOM_TEXT_STYLE: TextStyle = TextStyleBuilder::new().baseline(Baseline::Bottom).build();

fn localized_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Mo",
        Weekday::Tuesday => "Di",
        Weekday::Wednesday => "Mi",
        Weekday::Thursday => "Do",
        Weekday::Friday => "Fr",
        Weekday::Saturday => "Sa",
        Weekday::Sunday => "So",
    }
}

fn localized_month(month: i8) -> &'static str {
    match month {
        1 => "JAN",
        2 => "FEB",
        3 => "MÄR",
        4 => "APR",
        5 => "MAI",
        6 => "JUN",
        7 => "JUL",
        8 => "AUG",
        9 => "SEP",
        10 => "OKT",
        11 => "NOV",
        12 => "DEZ",
        _ => "???",
    }
}

pub fn draw_date(display: &mut Display7in5, date: Date) -> Result<(), Infallible> {
    let day: String<2> = format!("{:02}", date.day()).expect("formatting day");
    let month: String<2> = format!("{:02}", date.month()).expect("formatting month");

    Text::with_text_style(&day, Point::new(0, 0), SPLEEN_HUGE_STYLE, TOP_TEXT_STYLE)
        .draw(display)?;

    Text::with_text_style(&month, Point::new(0, 32), SPLEEN_HUGE_STYLE, TOP_TEXT_STYLE)
        .draw(display)?;

    Ok(())
}

pub fn draw_hourly_weather(
    display: &mut Display7in5,
    forecast: impl Iterator<Item = weather::HourlyForecast>,
) -> Result<(), Infallible> {
    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    for (index, forecast) in forecast.enumerate() {
        let temperature: String<8> = format!("{:.0}°C", fix_minus_zero(forecast.temperature))
            .expect("formatting temperature");

        let hour: String<8> = format!("{:0>2}:00", forecast.hour).unwrap();

        let x = (index as i32 + 1) * 72;

        Text::with_text_style(&hour, Point::new(x, 3), SPLEEN_SMALL_STYLE, text_style)
            .draw(display)?;

        Text::with_text_style(
            &temperature,
            Point::new(x, 54),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)?;

        Image::new(
            weather::hourly_icon(forecast.hour, forecast.weather_code),
            Point::new(x - 16, 17),
        )
        .draw(display)?;
    }

    Ok(())
}

pub fn draw_daily_weather(
    display: &mut Display7in5,
    forecast: impl Iterator<Item = weather::DailyForecast>,
) -> Result<(), Infallible> {
    Line::new(Point::new(3 * 72 + 39, 8), Point::new(3 * 72 + 39, 50))
        .into_styled(LINE_STYLE)
        .draw(display)?;

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    for (index, forecast) in forecast.enumerate() {
        let temperature: String<16> = format!(
            "{:.0}°C/{:.0}°C",
            fix_minus_zero(forecast.min_temperature),
            fix_minus_zero(forecast.max_temperature)
        )
        .expect("formatting temperature");

        let x = (index as i32 + 1) * 72 + 3 * 72 + 5;

        Text::with_text_style(
            localized_weekday(forecast.date.weekday()),
            Point::new(x, 3),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)?;

        Text::with_text_style(
            &temperature,
            Point::new(x, 54),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)?;

        Image::new(
            weather::hourly_icon(12, forecast.weather_code),
            Point::new(x - 16, 17),
        )
        .draw(display)?;
    }

    Ok(())
}

fn draw_vertical_month_label(
    display: &mut Display7in5,
    month: i8,
    start_y: i32,
    end_y: i32,
    x: i32,
) -> Result<(), Infallible> {
    let month_name = localized_month(month);
    let char_height = 16; // Height of SPLEEN_SMALL font
    let total_text_height = 3 * char_height; // We only have three-character abbreviated months
    let span_height = end_y - start_y;

    let text_start_y = start_y + (span_height - total_text_height) / 2;

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    for (index, char) in month_name.chars().enumerate() {
        let mut char_buf: String<4> = String::new();
        char_buf.push(char).ok();

        Text::with_text_style(
            &char_buf,
            Point::new(x, text_start_y + index as i32 * char_height),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)?;
    }

    Ok(())
}

pub fn draw_events(display: &mut Display7in5, events: &[ics::Event]) -> Result<(), Infallible> {
    const MONTH_COL_X: i32 = 6; // X position for month label (centered)
    const MONTH_LINE_X: i32 = 20; // X position for vertical month line
    const DAY_COL_X: i32 = 40; // X position for day column (shifted right)
    const EVENT_COL_X: i32 = 72; // X position for event details (shifted right)
    const EVENT_HEIGHT: i32 = 60;
    const FIRST_EVENT_Y: i32 = 78; // Y offset for first event

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    struct MonthGroup {
        month: i8,
        start_index: usize,
        end_index: usize,
    }

    let mut month_groups: heapless::Vec<MonthGroup, 12> = heapless::Vec::new();
    let mut current_month = events[0].start.date().month();
    let mut group_start = 0;

    for (index, event) in events.iter().enumerate() {
        let event_month = event.start.date().month();
        if event_month != current_month {
            month_groups
                .push(MonthGroup {
                    month: current_month,
                    start_index: group_start,
                    end_index: index - 1,
                })
                .ok();

            current_month = event_month;
            group_start = index;
        }
    }

    month_groups
        .push(MonthGroup {
            month: current_month,
            start_index: group_start,
            end_index: events.len() - 1,
        })
        .ok();

    for group in &month_groups {
        let start_y = FIRST_EVENT_Y + group.start_index as i32 * EVENT_HEIGHT;
        let end_y = FIRST_EVENT_Y + (group.end_index as i32 + 1) * EVENT_HEIGHT;

        Line::new(
            Point::new(MONTH_LINE_X, start_y + 14),
            Point::new(MONTH_LINE_X, end_y + 2),
        )
        .into_styled(LINE_STYLE)
        .draw(display)?;

        Line::new(
            Point::new(MONTH_LINE_X, start_y + 14),
            Point::new(MONTH_LINE_X + 4, start_y + 14),
        )
        .into_styled(LINE_STYLE)
        .draw(display)?;

        Line::new(
            Point::new(MONTH_LINE_X, end_y + 2),
            Point::new(MONTH_LINE_X + 4, end_y + 2),
        )
        .into_styled(LINE_STYLE)
        .draw(display)?;

        // Draw month label vertically centered
        draw_vertical_month_label(display, group.month, start_y + 12, end_y + 6, MONTH_COL_X);
    }

    for (index, event) in events.iter().enumerate() {
        let y_offset = FIRST_EVENT_Y + index as i32 * EVENT_HEIGHT;
        let day = strtime::format("%d", event.start.date()).unwrap();

        Text::with_text_style(
            &day,
            Point::new(DAY_COL_X, y_offset + 20),
            SPLEEN_LARGE_STYLE,
            text_style,
        )
        .draw(display)?;

        Text::with_text_style(
            localized_weekday(event.start.date().weekday()),
            Point::new(DAY_COL_X, y_offset + 46),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)?;

        // Draw event column with summary and times
        Text::with_text_style(
            &event.summary,
            Point::new(EVENT_COL_X, y_offset + 20),
            SPLEEN_LARGE_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)?;

        let (duration_x, duration): (i32, String<32>) = match (&event.start, &event.end) {
            (Either::Date(_), Either::Date(end_date)) => {
                let line_y = y_offset + 46 + 8;

                Line::new(
                    Point::new(EVENT_COL_X, line_y),
                    Point::new(EVENT_COL_X + 16, line_y),
                )
                .into_styled(LINE_STYLE)
                .draw(display)?;

                let end_formatted = format!(
                    "bis {}, {}",
                    localized_weekday(end_date.weekday()),
                    strtime::format("%d.%m.", *end_date).unwrap()
                )
                .unwrap();

                (EVENT_COL_X + 24, end_formatted)
            }
            _ => {
                let start = format_either(event.start.clone());
                let end = format_either(event.end.clone());
                (EVENT_COL_X, format!("{start} - {end}").unwrap())
            }
        };

        Text::with_text_style(
            &duration,
            Point::new(duration_x, y_offset + 46),
            SPLEEN_SMALL_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)?;
    }

    Ok(())
}

pub fn draw_todos<'a>(
    display: &mut Display7in5,
    todos: impl Iterator<Item = &'a str>,
) -> Result<(), Infallible> {
    let mut y = 790;

    for todo in todos
        .take(3)
        .collect::<heapless::Vec<_, 3>>()
        .into_iter()
        .rev()
    {
        RoundedRectangle::with_equal_corners(
            Rectangle::with_center(Point::new(6, y - 12), Size::new_equal(12)),
            Size::new_equal(4),
        )
        .into_styled(LINE_STYLE)
        .draw(display)?;

        Text::with_text_style(
            todo,
            Point::new(22, y),
            SPLEEN_LARGE_STYLE,
            BOTTOM_TEXT_STYLE,
        )
        .draw(display)?;

        y -= 28;
    }

    Ok(())
}

fn format_either(either: Either) -> String<16> {
    match either {
        ics::Either::DateTime(zoned) => {
            format!("{}", strtime::format("%H:%M", zoned.time()).unwrap())
                .expect("formatting zoned")
        }
        ics::Either::Date(date) => format!(
            "{}, {}",
            localized_weekday(date.weekday()),
            strtime::format("%d.%m", date).unwrap()
        )
        .expect("formatting date"),
    }
}

fn fix_minus_zero(num: f32) -> f32 {
    if num > -1.0 && num < 0.0 { 0.0 } else { num }
}

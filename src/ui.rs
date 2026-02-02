include!(concat!(env!("OUT_DIR"), "/font_spleen_8_16.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_12_24.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_16_32.rs"));

use embedded_graphics::image::Image;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::prelude::{Drawable, Point, Primitive};
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder};
use epd_waveshare::epd7in5_v2::Display7in5;
use epd_waveshare::prelude::*;
use esp_backtrace as _;
use heapless::{String, format};
use jiff::civil::{Date, Weekday};
use jiff::fmt::strtime;

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

pub fn draw_date(display: &mut Display7in5, date: Date) {
    let day: String<2> = format!("{:02}", date.day()).expect("formatting day");
    let month: String<2> = format!("{:02}", date.month()).expect("formatting month");

    Text::with_text_style(&day, Point::new(0, 0), SPLEEN_HUGE_STYLE, TOP_TEXT_STYLE)
        .draw(display)
        .unwrap();

    Text::with_text_style(&month, Point::new(0, 32), SPLEEN_HUGE_STYLE, TOP_TEXT_STYLE)
        .draw(display)
        .unwrap();
}

pub fn draw_hourly_weather(
    display: &mut Display7in5,
    forecast: impl Iterator<Item = weather::HourlyForecast>,
) {
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
            .draw(display)
            .unwrap();

        Text::with_text_style(
            &temperature,
            Point::new(x, 54),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        Image::new(
            weather::hourly_icon(forecast.hour, forecast.weather_code),
            Point::new(x - 16, 17),
        )
        .draw(display)
        .unwrap();
    }
}

pub fn draw_daily_weather(
    display: &mut Display7in5,
    forecast: impl Iterator<Item = weather::DailyForecast>,
) {
    Line::new(Point::new(3 * 72 + 39, 8), Point::new(3 * 72 + 39, 50))
        .into_styled(LINE_STYLE)
        .draw(display)
        .expect("drawing line");

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
        .draw(display)
        .unwrap();

        Text::with_text_style(
            &temperature,
            Point::new(x, 54),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        Image::new(
            weather::hourly_icon(12, forecast.weather_code),
            Point::new(x - 16, 17),
        )
        .draw(display)
        .unwrap();
    }
}

fn draw_vertical_month_label(
    display: &mut Display7in5,
    month: i8,
    start_y: i32,
    end_y: i32,
    x: i32,
) {
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
        .draw(display)
        .unwrap();
    }
}

pub fn draw_events(display: &mut Display7in5, events: &[ics::Event]) {
    if events.is_empty() {
        return;
    }

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    const MONTH_COL_X: i32 = 6; // X position for month label (centered)
    const MONTH_LINE_X: i32 = 20; // X position for vertical month line
    const DAY_COL_X: i32 = 40; // X position for day column (shifted right)
    const EVENT_COL_X: i32 = 72; // X position for event details (shifted right)
    const EVENT_HEIGHT: i32 = 64;
    const FIRST_EVENT_Y: i32 = 70; // Y offset for first event

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
        .draw(display)
        .expect("drawing month line");

        Line::new(
            Point::new(MONTH_LINE_X, start_y + 14),
            Point::new(MONTH_LINE_X + 4, start_y + 14),
        )
        .into_styled(LINE_STYLE)
        .draw(display)
        .unwrap();

        Line::new(
            Point::new(MONTH_LINE_X, end_y + 2),
            Point::new(MONTH_LINE_X + 4, end_y + 2),
        )
        .into_styled(LINE_STYLE)
        .draw(display)
        .unwrap();

        // Draw month label vertically centered
        draw_vertical_month_label(display, group.month, start_y + 12, end_y + 6, MONTH_COL_X);
    }

    for (index, event) in events.iter().enumerate() {
        let y_offset = FIRST_EVENT_Y + index as i32 * EVENT_HEIGHT;
        let start_date = event.start.date();

        let day = strtime::format("%d", start_date).unwrap();
        let start_time = strtime::format("%H:%M", event.start.time()).unwrap();
        let end_time = strtime::format("%H:%M", event.end.time()).unwrap();

        Text::with_text_style(
            &day,
            Point::new(DAY_COL_X, y_offset + 20),
            SPLEEN_LARGE_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        Text::with_text_style(
            localized_weekday(start_date.weekday()),
            Point::new(DAY_COL_X, y_offset + 46),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        // Draw event column with summary and times
        Text::with_text_style(
            &event.summary,
            Point::new(EVENT_COL_X, y_offset + 20),
            SPLEEN_LARGE_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)
        .unwrap();

        let duration: String<16> = format!("{start_time}-{end_time}").unwrap();

        Text::with_text_style(
            &duration,
            Point::new(EVENT_COL_X, y_offset + 46),
            SPLEEN_SMALL_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)
        .unwrap();
    }
}

fn fix_minus_zero(num: f32) -> f32 {
    if num > -1.0 && num < 0.0 { 0.0 } else { num }
}

include!(concat!(env!("OUT_DIR"), "/font_spleen_8_16.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_12_24.rs"));
include!(concat!(env!("OUT_DIR"), "/font_spleen_16_32.rs"));
include!(concat!(env!("OUT_DIR"), "/font_haxor_narrow_15.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/font_archivo_narrow_digits_36.rs"
));

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
        let temperature: String<8> =
            format!("{:.0}°C", forecast.temperature).expect("formatting temperature");

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
        .into_styled(PrimitiveStyle::with_stroke(Color::White, 1))
        .draw(display)
        .expect("drawing line");

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    for (index, forecast) in forecast.enumerate() {
        let temperature: String<10> = format!(
            "{:.0}°C/{:.0}°C",
            forecast.min_temperature, forecast.max_temperature
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

pub fn draw_events(display: &mut Display7in5, events: &[ics::Event]) {
    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Top)
        .build();

    for (index, event) in events.iter().enumerate() {
        let index = index + 1;
        let start_date = event.start.date();

        let day = strtime::format("%d", start_date).unwrap();
        let start_time = strtime::format("%H:%M", event.start.time()).unwrap();
        let end_time = strtime::format("%H:%M", event.end.time()).unwrap();

        // Draw first colum with day and week day.
        Text::with_text_style(
            &day,
            Point::new(16, index as i32 * 64 + 20),
            SPLEEN_LARGE_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        Text::with_text_style(
            localized_weekday(start_date.weekday()),
            Point::new(16, index as i32 * 64 + 46),
            SPLEEN_SMALL_STYLE,
            text_style,
        )
        .draw(display)
        .unwrap();

        // Draw second column with summary and times
        Text::with_text_style(
            &event.summary,
            Point::new(48, index as i32 * 64 + 20),
            SPLEEN_LARGE_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)
        .unwrap();

        let duration: String<16> = format!("{start_time}-{end_time}").unwrap();

        Text::with_text_style(
            &duration,
            Point::new(48, index as i32 * 64 + 46),
            SPLEEN_SMALL_STYLE,
            TOP_TEXT_STYLE,
        )
        .draw(display)
        .unwrap();
    }
}

use embedded_graphics::image::ImageRaw;
use embedded_nal_async::{Dns, TcpConnect};
use epd_waveshare::color::Color;
use reqwless::{client::HttpClient, request::Method};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::serde_as;

use crate::icons;

const HOURLY_URL: &str = "https://api.open-meteo.com/v1/forecast?latitude=49.0068901&longitude=8.4036527&hourly=temperature_2m,weather_code&timezone=Europe%2FBerlin&forecast_days=2";
const DAILY_URL: &str = "https://api.open-meteo.com/v1/forecast?latitude=49.0068901&longitude=8.4036527&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=Europe%2FBerlin&forecast_days=4";

#[derive(Debug)]
pub struct HourlyForecast {
    pub hour: u8,
    pub temperature: f32,
    pub weather_code: WeatherCode,
}

#[derive(Debug)]
pub struct DailyForecast {
    pub date: jiff::civil::Date,
    pub min_temperature: f32,
    pub max_temperature: f32,
    pub weather_code: WeatherCode,
}

#[derive(Deserialize)]
struct HourlyResponse {
    hourly: HourlyData,
}

#[serde_as]
#[derive(Deserialize)]
struct HourlyData {
    #[serde_as(as = "[_; 48]")]
    temperature_2m: [f32; 48],
    #[serde_as(as = "[_; 48]")]
    weather_code: [WeatherCode; 48],
}

#[derive(Deserialize)]
struct DailyResponse {
    daily: DailyData,
}

#[serde_as]
#[derive(Deserialize)]
struct DailyData {
    time: [jiff::civil::Date; 4],
    temperature_2m_max: [f32; 4],
    temperature_2m_min: [f32; 4],
    weather_code: [WeatherCode; 4],
}

pub async fn hourly_forecast<T, D>(
    client: &mut HttpClient<'_, T, D>,
) -> Option<heapless::Vec<HourlyForecast, 48>>
where
    T: TcpConnect,
    D: Dns,
{
    let mut write_buffer = [0u8; 1024];
    let mut read_buffer = [0u8; 4096 + 2048];

    let bytes_read = get(client, HOURLY_URL, &mut write_buffer, &mut read_buffer).await?;

    let response = match serde_json_core::from_slice::<HourlyResponse>(&read_buffer[..bytes_read]) {
        Ok((data, _)) => Some(data),
        Err(e) => {
            log::error!("Failed to parse weather response: {:?}", e);
            None
        }
    }?;

    Some(
        response
            .hourly
            .temperature_2m
            .into_iter()
            .zip(response.hourly.weather_code)
            .enumerate()
            .map(|(index, (temperature, weather_code))| HourlyForecast {
                hour: index as u8 % 24,
                temperature,
                weather_code,
            })
            .collect(),
    )
}

pub async fn daily_forecast<T, D>(
    client: &mut HttpClient<'_, T, D>,
) -> Option<heapless::Vec<DailyForecast, 4>>
where
    T: TcpConnect,
    D: Dns,
{
    let mut write_buffer = [0u8; 1024];
    let mut read_buffer = [0u8; 4096];

    let bytes_read = get(client, DAILY_URL, &mut write_buffer, &mut read_buffer).await?;

    let response = match serde_json_core::from_slice::<DailyResponse>(&read_buffer[..bytes_read]) {
        Ok((data, _)) => Some(data),
        Err(e) => {
            log::error!("Failed to parse weather response: {:?}", e);
            None
        }
    }?;

    let DailyData {
        time: date,
        temperature_2m_min: min_temperature,
        temperature_2m_max: max_temperature,
        weather_code,
    } = response.daily;

    Some(
        date.into_iter()
            .zip(min_temperature)
            .zip(max_temperature)
            .zip(weather_code)
            .map(
                |(((date, min_temperature), max_temperature), weather_code)| DailyForecast {
                    date,
                    min_temperature,
                    max_temperature,
                    weather_code,
                },
            )
            .collect(),
    )
}

async fn get<T, D>(
    client: &mut HttpClient<'_, T, D>,
    url: &str,
    write_buffer: &mut [u8],
    read_buffer: &mut [u8],
) -> Option<usize>
where
    T: TcpConnect,
    D: Dns,
{
    let size = client
        .request(Method::GET, url)
        .await
        .ok()?
        .send(write_buffer)
        .await
        .ok()?
        .body()
        .reader()
        .read_to_end(read_buffer)
        .await
        .ok()?;

    Some(size)
}

pub fn hourly_icon(hour: u8, weather_code: WeatherCode) -> &'static ImageRaw<'static, Color> {
    match (hour, weather_code) {
        (8..=19, WeatherCode::Clear | WeatherCode::MainlyClear) => &icons::SUN,
        (8..=19, WeatherCode::PartlyCloudy) => &icons::CLOUD_SUN,
        (0..=7 | 20..=23, WeatherCode::Clear | WeatherCode::MainlyClear) => &icons::MOON,
        (0..=7 | 20..=23, WeatherCode::PartlyCloudy) => &icons::CLOUD_MOON,
        (_, WeatherCode::Overcast) => &icons::CLOUD,
        (_, WeatherCode::Fog) => &icons::CLOUD_WIND,
        (_, WeatherCode::SlightRain | WeatherCode::LightDrizzle | WeatherCode::ModerateDrizzle) => {
            &icons::RAIN0
        }
        (_, WeatherCode::ModerateRain | WeatherCode::SlightRainShower) => &icons::RAIN1,
        (_, WeatherCode::HeavyRain) => &icons::RAIN2,
        x => {
            log::warn!("{x:?} not covered");
            &icons::SUN
        }
    }
}

#[derive(Copy, Clone, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum WeatherCode {
    Clear = 0,
    MainlyClear = 1,
    PartlyCloudy = 2,
    Overcast = 3,
    Fog = 45,
    DepositingRimeFog = 48,
    LightDrizzle = 51,
    ModerateDrizzle = 53,
    DenseDrizzle = 55,
    LightFreezingDrizzle = 56,
    DenseFreezingDrizzle = 57,
    SlightRain = 61,
    ModerateRain = 63,
    HeavyRain = 65,
    FreezingLightRain = 66,
    FreezingHeavyRain = 67,
    SlightSnow = 71,
    ModerateSnow = 73,
    HeavySnow = 75,
    SnowGrains = 77,
    SlightRainShower = 80,
    ModerateRainShower = 81,
    ViolentRainShower = 82,
    SlightThunderstorm = 95,
    SlightThunderstormSlightHail = 96,
    SlightThunderstormHeavyHail = 99,
}

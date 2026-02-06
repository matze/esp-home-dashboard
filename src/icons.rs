//! Auto-generated 1-bit raw icon data for embedded_graphics.
//!
//! Usage with embedded_graphics:
//! ```rust
//! use embedded_graphics::{
//!     image::{Image, ImageRaw},
//!     pixelcolor::BinaryColor,
//!     prelude::*,
//! };
//!
//! let raw_image: ImageRaw<BinaryColor> = ImageRaw::new(CLOUD, 32);
//! let image = Image::new(&raw_image, Point::zero());
//! image.draw(&mut display)?;
//! ```

use embedded_graphics::image::ImageRaw;
use epd_waveshare::color::Color;

pub const CLOUD: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/cloud.raw"), 32);

pub const CLOUD_MOON: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/cloud_moon.raw"), 32);

pub const CLOUD_SUN: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/cloud_sun.raw"), 32);

pub const CLOUD_WIND: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/cloud_wind.raw"), 32);

pub const CLOUD_WIND_MOON: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/cloud_wind_moon.raw"), 32);

pub const CLOUD_WIND_SUN: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/cloud_wind_sun.raw"), 32);

pub const CLOUDS: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/clouds.raw"), 32);

pub const LIGHTNING: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/lightning.raw"), 32);

pub const MOON: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/moon.raw"), 32);

pub const RAIN0: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/rain0.raw"), 32);

pub const RAIN0_SUN: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/rain0_sun.raw"), 32);

pub const RAIN1: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/rain1.raw"), 32);

pub const RAIN1_MOON: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/rain1_moon.raw"), 32);

pub const RAIN1_SUN: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/rain1_sun.raw"), 32);

pub const RAIN2: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/rain2.raw"), 32);

pub const RAIN_LIGHTNING: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/rain_lightning.raw"), 32);

pub const RAIN_SNOW: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/rain_snow.raw"), 32);

pub const SNOW: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/snow.raw"), 32);

pub const SNOW_MOON: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/snow_moon.raw"), 32);

pub const SNOW_SUN: ImageRaw<Color> =
    ImageRaw::new(include_bytes!("../assets/icons/snow_sun.raw"), 32);

pub const SUN: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/sun.raw"), 32);

pub const WIND: ImageRaw<Color> = ImageRaw::new(include_bytes!("../assets/icons/wind.raw"), 32);

/// 16x16 icon spritesheet (6 columns x 8 rows = 48 icons, tile-ordered).
const PYAIRVANDER_ICON_DATA: &[u8] = include_bytes!("../assets/icons/1bit_fantasy_pyairvander.raw");

const PYAIRVANDER_BYTES_PER_ICON: usize = 16 * 16 / 8; // 32

pub const PYAIRVANDER_ICON_COUNT: usize = PYAIRVANDER_ICON_DATA.len() / PYAIRVANDER_BYTES_PER_ICON;

/// Returns a 16x16 icon from the pyairvander spritesheet by index (0..48).
///
/// # Panics
///
/// Panics if `index >= 48`.
pub fn pyairvander_icon(index: usize) -> ImageRaw<'static, Color> {
    assert!(index < PYAIRVANDER_ICON_COUNT);
    let offset = index * PYAIRVANDER_BYTES_PER_ICON;
    ImageRaw::new(
        &PYAIRVANDER_ICON_DATA[offset..offset + PYAIRVANDER_BYTES_PER_ICON],
        16,
    )
}

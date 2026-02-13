#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_futures::join;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{DhcpConfig, dns::DnsSocket};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::prelude::DrawTarget;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::epd7in5_v2::{Display7in5, Epd7in5};
use epd_waveshare::prelude::*;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::rng::Rng;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::timer::timg::TimerGroup;
use reqwless::client::{HttpClient, TlsConfig};

use esp_home_dashboard::{clock, ics, ntp, todo, ui, weather, wifi};

esp_bootloader_esp_idf::esp_app_desc!();

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const ICAL_URL: &str = env!("ICAL_URL");
const TIMEZONE_DATA_EUROPE_BERLIN: &[u8] = include_bytes!("/usr/share/zoneinfo/Europe/Berlin");
const NTP_HOST_NAME: Option<&str> = option_env!("NTP_HOST_NAME");
const TODO_URL: Option<&str> = option_env!("TODO_URL");
const TODO_AUTHORIZATION_HEADER: Option<&str> = option_env!("TODO_AUTHORIZATION_HEADER");

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(esp_hal::time::Rate::from_mhz(10))
            .with_mode(Mode::_0),
    )
    .expect("creating Spi")
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7)
    .into_async();

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let busy = Input::new(
        peripherals.GPIO1,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let dc = Output::new(peripherals.GPIO0, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO3, Level::High, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());

    let mut spi = ExclusiveDevice::new(spi, cs, Delay).expect("creating SPI device");
    let mut epd = Epd7in5::new(&mut spi, busy, dc, rst, &mut Delay, None)
        .await
        .expect("creating EPD");

    let mut display = Display7in5::default();

    display.set_rotation(DisplayRotation::Rotate90);

    let radio_init = esp_radio::init().expect("initializing Wi-Fi/BLE controller");

    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("initializing Wi-Fi controller");
    let wifi_device = interfaces.sta;

    let rng = Rng::new();
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let net_config = embassy_net::Config::dhcpv4(DhcpConfig::default());

    // Need these large TLS buffers to reliably connect to iCal servers.
    let mut tls_read_buffer = [0; 16384];
    let mut tls_write_buffer = [0; 16384];

    let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let timezone = jiff::tz::TimeZone::tzif("Europe/Berlin", TIMEZONE_DATA_EUROPE_BERLIN)
        .expect("parsing timezone data");

    let clock = clock::Clock::new(timezone.clone());

    // Careful: this needs to cover _all_ sockets we want to use.
    let mut resources = embassy_net::StackResources::<4>::new();

    let (net_stack, mut net_runner) =
        embassy_net::new(wifi_device, net_config, &mut resources, net_seed);

    let sync_time = ntp::sync(
        &net_stack,
        NTP_HOST_NAME.unwrap_or("de.pool.ntp.org"),
        clock.clone(),
    );

    let main_logic = async {
        loop {
            log::debug!("waiting for link");

            net_stack.wait_link_up().await;
            net_stack.wait_config_up().await;

            let dns = DnsSocket::new(net_stack);
            let tcp_state = TcpClientState::<3, 4096, 4096>::new();
            let tcp = TcpClient::new(net_stack, &tcp_state);

            let tls_config = TlsConfig::new(
                tls_seed,
                &mut tls_read_buffer,
                &mut tls_write_buffer,
                reqwless::client::TlsVerify::None,
            );

            let mut client = HttpClient::new_with_tls(&tcp, &dns, tls_config);

            display.clear(Color::Black);

            match weather::hourly_forecast(&mut client).await {
                Ok(forecast) => {
                    let hour = clock.now().time().hour();
                    let forecast = forecast.into_iter().skip(hour as usize).step_by(2).take(3);

                    ui::draw_hourly_weather(&mut display, forecast);
                }
                Err(err) => {
                    log::error!("failed to fetch hourly forecast: {err:?}");
                }
            }

            match weather::daily_forecast(&mut client).await {
                Ok(forecast) => {
                    let forecast = forecast.into_iter().skip(1);
                    ui::draw_daily_weather(&mut display, forecast);
                }
                Err(err) => {
                    log::error!("failed to fetch daily forecast: {err:?}");
                }
            }

            ui::draw_date(&mut display, clock.now().date());

            let mut events: [ics::Event; 10] = Default::default();

            match ics::get_events(&mut client, clock.clone(), ICAL_URL, &mut events).await {
                Ok(events) => {
                    ui::draw_events(&mut display, events, clock.now().date());
                }
                Err(err) => {
                    log::error!("failed to fetch events: {err:?}");
                }
            }

            if let Some((url, auth_header)) = TODO_URL.zip(TODO_AUTHORIZATION_HEADER) {
                let mut read_buffer = [0u8; 1024];

                match todo::get_todos(&mut client, url, auth_header, &mut read_buffer).await {
                    Ok(todos) => {
                        ui::draw_todos(&mut display, todos);
                    }
                    Err(err) => {
                        log::error!("failed to fetch todos: {err:?}");
                    }
                }
            }

            epd.wake_up(&mut spi, &mut Delay)
                .await
                .expect("waking up the display");

            epd.update_and_display_frame(&mut spi, display.buffer(), &mut Delay)
                .await
                .expect("failed to display frame");

            // After DisplayRefresh the display needs time to start the
            // refresh and assert BUSY. Without this delay wait_until_idle
            // can see BUSY still de-asserted and return immediately.
            Timer::after(Duration::from_secs(1)).await;

            epd.wait_until_idle(&mut spi, &mut Delay)
                .await
                .expect("wait until idle to succeed");

            epd.sleep(&mut spi, &mut Delay)
                .await
                .expect("failed to put EPD to sleep");

            // Schedule next update for the next full hour. Add a minute for some leeway.
            Timer::after(Duration::from_secs(60 * (61 - clock.now().minute() as u64))).await;
        }
    };

    join::join(
        net_runner.run(),
        join::join3(
            wifi::keep_connection(wifi_controller, WIFI_SSID, WIFI_PASSWORD),
            main_logic,
            sync_time,
        ),
    )
    .await;

    log::error!("all futures finished ...");

    // We should not end up here but need it without the Never type.
    loop {
        log::info!("invalid state");
        Timer::after(Duration::from_secs(5)).await;
    }
}

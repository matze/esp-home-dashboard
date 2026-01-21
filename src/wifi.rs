use embassy_time::{Duration, Timer};
use esp_radio::wifi::{ClientConfig, ModeConfig, WifiController, WifiEvent, WifiStaState};

pub async fn keep_connection(mut controller: WifiController<'_>, ssid: &str, password: &str) {
    loop {
        if matches!(esp_radio::wifi::sta_state(), WifiStaState::Connected) {
            log::info!("connected to Wi-Fi");
            controller.wait_for_event(WifiEvent::StaDisconnected).await;

            log::warn!("disconnected from Wi-Fi");
            Timer::after(Duration::from_secs(5)).await;
        }

        let Ok(is_started) = controller.is_started() else {
            continue;
        };

        if !is_started {
            let config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(ssid.into())
                    .with_password(password.into()),
            );

            controller
                .set_config(&config)
                .expect("setting Wi-Fi client config");

            log::info!("starting Wi-Fi");
            controller.start_async().await.expect("starting Wi-Fi");
        }

        match controller.connect_async().await {
            Ok(()) => log::info!("connected to {ssid}"),
            Err(err) => {
                log::error!("failed to connect to Wi-Fi: {err:?}");
                Timer::after(Duration::from_secs(5)).await
            }
        }
    }
}

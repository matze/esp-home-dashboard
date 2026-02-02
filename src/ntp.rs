use embassy_net::{Stack, udp::UdpSocket};
use embassy_time::{Instant, Timer};
use sntpc_net_embassy::UdpSocketWrapper;

use crate::clock::Clock;

#[derive(Copy, Clone, Default)]
struct TimestampGenerator {
    duration: u64,
}

impl sntpc::NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {
        self.duration = Instant::now().as_micros();
    }

    fn timestamp_sec(&self) -> u64 {
        self.duration / 1_000_000
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        (self.duration % 1_000_000) as u32
    }
}

pub async fn sync(net_stack: &Stack<'_>, host_name: &str, clock: Clock) {
    loop {
        net_stack.wait_link_up().await;
        net_stack.wait_config_up().await;

        let mut rx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
        let mut rx_buffer = [0u8; 512];
        let mut tx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
        let mut tx_buffer = [0u8; 512];

        let mut socket = UdpSocket::new(
            *net_stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        if let Err(err) = socket.bind(0) {
            log::error!("failed to bind UDP socket for NTP: {err:?}");
            return;
        }

        let addresses = match net_stack
            .dns_query(host_name, embassy_net::dns::DnsQueryType::A)
            .await
        {
            Ok(addresses) => addresses,
            Err(err) => {
                log::error!("failed to query DNS record: {err:?}");
                Timer::after_secs(30).await;
                continue;
            }
        };

        log::info!("resolved NTP name to {addresses:?}");

        if let Some(address) = addresses.first() {
            let socket_wrapper = UdpSocketWrapper::new(socket);
            let context = sntpc::NtpContext::new(TimestampGenerator::default());

            match sntpc::get_time((*address, 123).into(), &socket_wrapper, context).await {
                Ok(time) => {
                    clock.sync(time.sec().into());
                    Timer::after_secs(3600).await;
                }
                Err(e) => log::error!("NTP error: {:?}", e),
            }
        }
    }
}

use anyhow::Context;
use metrics_runtime::Receiver;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::env;

pub use metrics::{counter, gauge, timing, value};

const METRICS_PORT: u16 = 51805;

pub fn serve() {
    tokio::spawn(setup_metrics());
}

async fn setup_metrics() -> anyhow::Result<()> {
    let metrics_receiver = Receiver::builder()
        .build()
        .context("failed to create receiver")?;
    let controller = metrics_receiver.controller();
    metrics_receiver.install();

    let port : u16 = match env::var("CDL_METRICS") {
        Ok(o) => o.parse::<u16>()?,
        Err(_) => METRICS_PORT,
    };

    let metrics_exporter = metrics_exporter_http::HttpExporter::new(
        controller,
        metrics_observer_prometheus::PrometheusBuilder::new(),
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port).into(),
    );

    metrics_exporter
        .async_run()
        .await
        .context("Failed to serve metrics")
}

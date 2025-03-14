use std::io::Cursor;

use scuffle_flv::{audio::AudioData, video::VideoData};
use tokio::net::TcpListener;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Handler;

impl scuffle_rtmp::SessionHandler for Handler {
    async fn on_data(
        &self,
        _stream_id: u32,
        data: scuffle_rtmp::SessionData,
    ) -> Result<(), scuffle_rtmp::SessionError> {
        match data {
            scuffle_rtmp::SessionData::Audio { data, .. } => {
                let tag = AudioData::demux(&mut Cursor::new(data)).unwrap();
                tracing::info!("audio: {:?}", tag);
            }
            scuffle_rtmp::SessionData::Video { data, .. } => {
                let tag = VideoData::demux(&mut Cursor::new(data)).unwrap();
                tracing::info!("video: {:?}", tag);
            }
            scuffle_rtmp::SessionData::Amf0 { data, timestamp } => {
                tracing::info!("amf0 data, timestamp: {timestamp}, data: {data:?}");
            }
        }

        Ok(())
    }

    async fn on_publish(
        &self,
        stream_id: u32,
        app_name: &str,
        stream_name: &str,
    ) -> Result<(), scuffle_rtmp::SessionError> {
        tracing::info!(
            "publish, stream_id: {stream_id}, app_name: {app_name}, stream_name: {stream_name}"
        );
        Ok(())
    }

    async fn on_unpublish(&self, stream_id: u32) -> Result<(), scuffle_rtmp::SessionError> {
        tracing::info!("unpublish, stream_id: {stream_id}");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let listener = TcpListener::bind("[::]:1935").await.unwrap();
    tracing::info!("listening on [::]:1935");

    while let Ok((stream, addr)) = listener.accept().await {
        tracing::info!("accepted connection from {addr}");

        let mut session = scuffle_rtmp::Session::new(stream, Handler);
        tokio::spawn(async move {
            if let Err(err) = session
                .run()
                .instrument(tracing::info_span!("session", addr = %addr))
                .await
            {
                tracing::error!("session error: {:?}", err);
            }
        });
    }
}

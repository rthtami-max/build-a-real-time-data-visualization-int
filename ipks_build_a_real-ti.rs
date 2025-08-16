use tokio::prelude::*;
use tokio::stream::{self, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio::time;

struct RealtimeData {
    timestamp: u64,
    value: f64,
}

#[derive(Debug)]
enum IntegratorMessage {
    AddData(RealtimeData),
    GetStats(oneshot::Sender<Vec<RealtimeData>>),
}

struct Integrator {
    data: Vec<RealtimeData>,
    rx: mpsc::UnboundedReceiver<IntegratorMessage>,
}

impl Integrator {
    async fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { data: vec![], rx }
    }

    async fn run(self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                IntegratorMessage::AddData(data) => {
                    println!("Received data: {:?}", data);
                    self.data.push(data);
                }
                IntegratorMessage::GetStats(resp) => {
                    let stats = calculate_stats(&self.data);
                    resp.send(stats).unwrap();
                }
            }
        }
    }
}

fn calculate_stats(data: &[RealtimeData]) -> Vec<RealtimeData> {
    // implement calculation of statistics (e.g. mean, min, max, etc.)
    data.to_vec()
}

#[tokio::main]
async fn main() {
    let integrator = Integrator::new().await;
    let mut tx = integrator.rx.clone();

    tokio::spawn(async move {
        integrator.run().await;
    });

    let data = RealtimeData {
        timestamp: 1,
        value: 10.0,
    };
    tx.send(IntegratorMessage::AddData(data)).await.unwrap();

    let data = RealtimeData {
        timestamp: 2,
        value: 20.0,
    };
    tx.send(IntegratorMessage::AddData(data)).await.unwrap();

    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(IntegratorMessage::GetStats(resp_tx)).await.unwrap();

    let stats = resp_rx.await.unwrap();
    println!("Stats: {:?}", stats);
}
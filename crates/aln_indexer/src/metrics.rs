use prometheus::{Encoder, TextEncoder, IntCounter, IntGauge, register_int_counter, register_int_gauge};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Metrics {
    pub indexed_blocks_total: IntCounter,
    pub reorg_events_total: IntCounter,
    pub replayed_blocks_total: IntCounter,
    pub indexer_head_height: IntGauge,
    pub last_compacted_height: IntGauge,
}

impl Metrics {
    pub fn new() -> Arc<RwLock<Self>> {
        let indexed_blocks_total = register_int_counter!("indexed_blocks_total", "Total indexed blocks").unwrap();
        let reorg_events_total = register_int_counter!("reorg_events_total", "Reorg events encountered").unwrap();
        let replayed_blocks_total = register_int_counter!("replayed_blocks_total", "Replayed blocks during reindex").unwrap();
        let indexer_head_height = register_int_gauge!("indexer_head_height", "Indexer head height").unwrap();
        let last_compacted_height = register_int_gauge!("last_compacted_height", "Last compacted height").unwrap();
        Arc::new(RwLock::new(Self { indexed_blocks_total, reorg_events_total, replayed_blocks_total, indexer_head_height, last_compacted_height }))
    }

    pub async fn gather(self: Arc<RwLock<Self>>) -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

use prometheus::{Encoder, TextEncoder, IntCounter, IntGauge, IntCounterVec, IntGaugeVec, register_int_counter, register_int_gauge, register_int_counter_vec, register_int_gauge_vec};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Metrics {
    pub indexed_blocks_total: IntCounter,
    pub reorg_events_total: IntCounter,
    pub replayed_blocks_total: IntCounter,
    pub indexer_head_height: IntGauge,
    pub last_compacted_height: IntGauge,
    pub aln_bridge_events_total: IntCounter,
    pub aln_energy_toxic_total: IntGauge,
    pub aln_energy_clean_total: IntGauge,
    pub class_mint_total: IntCounterVec,
    pub class_burn_total: IntCounterVec,
    pub class_toxic_gauge: IntGaugeVec,
    pub sealed_refactor_total: IntCounter,
    pub sealed_refactor_rejected_total: IntCounter,
}

impl Metrics {
    pub fn new() -> Arc<RwLock<Self>> {
        let indexed_blocks_total = register_int_counter!("indexed_blocks_total", "Total indexed blocks").unwrap();
        let reorg_events_total = register_int_counter!("reorg_events_total", "Reorg events encountered").unwrap();
        let replayed_blocks_total = register_int_counter!("replayed_blocks_total", "Replayed blocks during reindex").unwrap();
        let indexer_head_height = register_int_gauge!("indexer_head_height", "Indexer head height").unwrap();
        let last_compacted_height = register_int_gauge!("last_compacted_height", "Last compacted height").unwrap();
        let aln_bridge_events_total = register_int_counter!("aln_bridge_events_total", "Total bridge events processed").unwrap();
        let aln_energy_toxic_total = register_int_gauge!("aln_energy_toxic_total", "Total toxic energy currently indexed").unwrap();
        let aln_energy_clean_total = register_int_gauge!("aln_energy_clean_total", "Total clean energy currently indexed").unwrap();
        let class_mint_total = register_int_counter_vec!("class_mint_total", "Total minted per class", &["class_id"]).unwrap();
        let class_burn_total = register_int_counter_vec!("class_burn_total", "Total burned per class", &["class_id"]).unwrap();
        let class_toxic_gauge = register_int_gauge_vec!("class_toxic_gauge", "Toxic flag per class", &["class_id"]).unwrap();
        let sealed_refactor_total = register_int_counter!("sealed_refactor_total", "Total sealed_refactor invocations processed").unwrap();
        let sealed_refactor_rejected_total = register_int_counter!("sealed_refactor_rejected_total", "Total sealed_refactor rejected by UBS").unwrap();
        Arc::new(RwLock::new(Self { indexed_blocks_total, reorg_events_total, replayed_blocks_total, indexer_head_height, last_compacted_height, aln_bridge_events_total, aln_energy_toxic_total, aln_energy_clean_total, class_mint_total, class_burn_total, class_toxic_gauge, sealed_refactor_total, sealed_refactor_rejected_total }))
    }

    pub async fn gather(self: Arc<RwLock<Self>>) -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub pass_count: u64,
    pub block_count: u64,
    pub block_rate: f64,
    pub latency_p50_ms: u128,
    pub latency_p95_ms: u128,
    pub top_block_reasons: Vec<ReasonHit>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReasonHit {
    pub reason: String,
    pub count: u64,
}

#[derive(Debug)]
struct MetricsState {
    pass_count: u64,
    block_count: u64,
    block_reasons: HashMap<String, u64>,
    latencies_ms: VecDeque<u128>,
}

#[derive(Debug)]
pub struct RuntimeMetrics {
    state: Mutex<MetricsState>,
    latency_window_size: usize,
}

impl RuntimeMetrics {
    pub fn new(latency_window_size: usize) -> Self {
        Self {
            state: Mutex::new(MetricsState {
                pass_count: 0,
                block_count: 0,
                block_reasons: HashMap::new(),
                latencies_ms: VecDeque::with_capacity(latency_window_size),
            }),
            latency_window_size,
        }
    }

    pub fn record_validation(&self, is_safe: bool, reason: &str, latency_ms: u128) {
        let mut guard = self.state.lock().expect("metrics mutex poisoned");

        if is_safe {
            guard.pass_count += 1;
        } else {
            guard.block_count += 1;
            *guard.block_reasons.entry(reason.to_string()).or_insert(0) += 1;
        }

        guard.latencies_ms.push_back(latency_ms);
        while guard.latencies_ms.len() > self.latency_window_size {
            guard.latencies_ms.pop_front();
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let guard = self.state.lock().expect("metrics mutex poisoned");

        let total_requests = guard.pass_count + guard.block_count;
        let block_rate = if total_requests == 0 {
            0.0
        } else {
            (guard.block_count as f64) / (total_requests as f64)
        };

        let mut sorted_reasons = guard
            .block_reasons
            .iter()
            .map(|(reason, count)| ReasonHit {
                reason: reason.to_string(),
                count: *count,
            })
            .collect::<Vec<_>>();
        sorted_reasons.sort_by_key(|hit| Reverse(hit.count));
        sorted_reasons.truncate(5);

        let mut samples = guard.latencies_ms.iter().copied().collect::<Vec<_>>();
        samples.sort_unstable();

        MetricsSnapshot {
            total_requests,
            pass_count: guard.pass_count,
            block_count: guard.block_count,
            block_rate,
            latency_p50_ms: percentile(&samples, 0.50),
            latency_p95_ms: percentile(&samples, 0.95),
            top_block_reasons: sorted_reasons,
        }
    }
}

fn percentile(sorted_values: &[u128], ratio: f64) -> u128 {
    if sorted_values.is_empty() {
        return 0;
    }

    let position = ((sorted_values.len() as f64) * ratio).ceil() as usize;
    let index = position.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values[index]
}

#[cfg(test)]
mod tests {
    use super::RuntimeMetrics;

    #[test]
    fn records_counts_and_block_reasons() {
        let metrics = RuntimeMetrics::new(64);
        metrics.record_validation(true, "PASS", 5);
        metrics.record_validation(false, "BLOCK_DENY_PATTERN", 8);
        metrics.record_validation(false, "BLOCK_DENY_PATTERN", 10);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 3);
        assert_eq!(snapshot.pass_count, 1);
        assert_eq!(snapshot.block_count, 2);
        assert!(snapshot.block_rate > 0.66 && snapshot.block_rate < 0.67);
        assert_eq!(snapshot.top_block_reasons[0].reason, "BLOCK_DENY_PATTERN");
        assert_eq!(snapshot.top_block_reasons[0].count, 2);
    }

    #[test]
    fn calculates_latency_percentiles_from_recent_window() {
        let metrics = RuntimeMetrics::new(4);
        for value in [10_u128, 20, 30, 40, 100] {
            metrics.record_validation(true, "PASS", value);
        }

        let snapshot = metrics.snapshot();
        // 10 is out due to window limit, samples become [20,30,40,100]
        assert_eq!(snapshot.latency_p50_ms, 30);
        assert_eq!(snapshot.latency_p95_ms, 100);
    }
}

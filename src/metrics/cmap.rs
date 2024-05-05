use anyhow::Result;
use dashmap::DashMap;
use std::{collections::HashMap, ops::Deref, sync::Arc};

///
/// concurrency metrics table
/// inc/dec/snaphot
///
/// - Arc<RwLock<HashMap<K, V>>>>
/// - DashMap
/// - AtomicXXX
/// - 共享内存处理, Send/Sync
/// - fearless concurrency
///

type MetricsTable = DashMap<String, i64>;

pub struct MetricsState {
    pub table: MetricsTable,
}

#[derive(Clone)]
pub struct Metrics(Arc<MetricsState>);

impl Deref for Metrics {
    type Target = MetricsState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self(Arc::new(MetricsState {
            table: MetricsTable::default(),
        }))
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<i64> {
        let key = key.into();
        let mut entry = self.table.entry(key.clone()).or_default();
        let val = entry.value_mut();
        *val += 1;
        Ok(*val)
    }

    pub fn dec(&self, key: impl Into<String>) -> Result<i64> {
        let key = key.into();
        let mut entry = self.table.entry(key.clone()).or_default();
        let val = entry.value_mut();
        *val -= 1;
        Ok(*val)
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .table
            .iter()
            .map(|v| (v.key().clone(), *v.value()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_metrics() {
        let metrics = Metrics::new();

        let mut incs = Vec::new();
        for _ in 0..6 {
            let metrics = metrics.clone();
            incs.push(thread::spawn(move || {
                for _ in 0..20 {
                    metrics.inc("test").unwrap();
                    thread::sleep(Duration::from_millis(rand::random::<u64>() % 50 + 20));
                }
            }));
        }

        let mut decs = Vec::new();
        for _ in 0..4 {
            let metrics = metrics.clone();
            decs.push(thread::spawn(move || {
                for _ in 0..20 {
                    metrics.dec("test").unwrap();
                    thread::sleep(Duration::from_millis(rand::random::<u64>() % 50 + 20));
                }
            }));
        }

        let mut readers = Vec::new();
        for _ in 0..20 {
            let metrics = metrics.clone();
            readers.push(thread::spawn(move || {
                for _ in 0..20 {
                    thread::sleep(Duration::from_millis(rand::random::<u64>() % 50 + 30));
                    assert!(metrics.snapshot().is_ok());
                }
            }));
        }

        for inc in incs {
            inc.join().unwrap();
        }
        for dec in decs {
            dec.join().unwrap();
        }
        for reader in readers {
            reader.join().unwrap();
        }

        let snapshot = metrics.snapshot();
        assert!(snapshot.is_ok());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.get("test"), Some(&40));
    }
}

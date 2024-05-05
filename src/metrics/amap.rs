use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

type MetricsTable = HashMap<&'static str, AtomicI64>;

pub struct AMetricsState {
    pub table: MetricsTable,
}

#[derive(Clone)]
pub struct AMetrics(Arc<AMetricsState>);

impl Deref for AMetrics {
    type Target = AMetricsState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AMetrics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.0).expect("AMetrics is not unique")
    }
}

impl AMetrics {
    pub fn new(keys: &[&'static str]) -> Self {
        let mut table = MetricsTable::new();
        for &key in keys {
            table.insert(key, AtomicI64::new(0));
        }
        Self(Arc::new(AMetricsState { table }))
    }

    pub fn inc(&self, key: &str) -> Result<i64> {
        let val = self
            .table
            .get(key)
            .context(format!("key {key} not found"))?;
        let val = val.fetch_add(1, Ordering::Relaxed);
        Ok(val)
    }

    pub fn dec(&self, key: &str) -> Result<i64> {
        let val = self
            .table
            .get(key)
            .context(format!("key {key} not found"))?;
        let val = val.fetch_sub(1, Ordering::Relaxed);
        Ok(val)
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .table
            .iter()
            .map(|(&k, v)| (k.to_owned(), v.load(Ordering::Relaxed)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_metrics() {
        let metrics = AMetrics::new(&["test"]);

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

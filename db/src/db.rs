use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use tokio::sync::{RwLock, Semaphore};

pub(crate) struct Db {
    shared: RwLock<DbState>,
}

struct DbState {
    keyspace: Keyspace,
    partitions: HashMap<String, Partition>,
}

struct Partition {
    handler: PartitionHandle,
    write_lock: Semaphore,
}

impl Db {
    pub(crate) async fn open_partition(&self, name: String) -> crate::Result<()> {
        let mut state = self.shared.write().await;

        let partition = state.keyspace
            .open_partition(&name,
                            PartitionCreateOptions::default()
                                .level_count(255),
            )
            .unwrap();

        state.partitions.insert(name.to_string(), Partition {
            handler: partition,
            write_lock: Semaphore::new(1),
        });

        drop(state);
        Ok(())
    }
    pub(crate) async fn write_to_partition(&self, partition_name: String, key: String, data: Vec<u8>, unlock: bool) -> crate::Result<()> {
        let state = self.shared.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();

        if unlock {
            partition.handler.insert(&key, data)?;
            partition.write_lock.add_permits(1);
            return Ok(());
        }

        {
            let _ = partition.write_lock.acquire().await.unwrap();
            partition.handler.insert(&key, data)?;
        }

        drop(state);
        Ok(())
    }
    pub(crate) async fn unlock(&self, partition_name: String) {
        {
            let state = self.shared.read().await;
            let partition = state.partitions.get(&partition_name).unwrap();
            partition.write_lock.add_permits(1);
        }
    }

    pub(crate) async fn read_from_partition(&self, partition_name: String, key: String) -> crate::Result<Vec<u8>> {
        let state = self.shared.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();
        let value = partition.handler.get(&key).unwrap().unwrap_or(Arc::new([]));
        drop(state);
        Ok(value.to_vec())
    }
    pub(crate) async fn read_latest_from_partition(&self, partition_name: String, lock: bool) -> crate::Result<Vec<u8>> {
        let state = self.shared.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();

        if lock {
            partition.write_lock.acquire().await.unwrap().forget();
        }

        let value = match partition.handler.last_key_value().unwrap() {
            Some((_, v)) => v,
            None => Arc::new([]),
        };
        drop(state);
        Ok(value.to_vec())
    }
    pub(crate) async fn read_last_from_partition(&self, partition_name: String, count: u16) -> crate::Result<Vec<Vec<u8>>> {
        let state = self.shared.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();
        let values = partition.handler.iter().into_iter().rev().take(count as usize)
            .map(|item| item.unwrap().1.to_vec())
            .collect::<Vec<_>>();
        drop(state);
        Ok(values)
    }
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> Db {
        Db {
            shared: RwLock::new(DbState {
                keyspace: Config::new(path)
                    .fsync_ms(Some(30_000))
                    .block_cache(Arc::new(fjall::BlockCache::with_capacity_bytes(/* 32 MiB */ 32 * 1_024 * 1_024)))
                    .compaction_workers(1)
                    .open().unwrap(),
                partitions: HashMap::new(),
            })
        }
    }
}
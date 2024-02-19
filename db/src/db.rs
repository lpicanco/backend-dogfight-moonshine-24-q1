use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc};
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use log::info;
use tokio::sync::RwLock;

#[derive(Clone)]
pub(crate) struct Db {
    shared: Arc<SharedState>,
    // state: Mutex<DbState>,
    // data: HashMap<String, String>,
    // keyspace: Keyspace,
    // partitions: HashMap<String, PartitionHandle>,
}

struct SharedState {
    state: RwLock<DbState>
}

struct DbState {
    keyspace: Keyspace,
    partitions: HashMap<String, PartitionHandle>,
}

impl Db {
    pub(crate) async fn open_partition(&self, name: String) -> crate::Result<()> {
        let mut state = self.shared.state.write().await;
        // let mut state = self.shared.state.write().unwrap();

        let partition = state.keyspace
            .open_partition(&name, PartitionCreateOptions::default().level_count(50))
            .unwrap();
        state.partitions.insert(name.to_string(), partition);

        drop(state);
        Ok(())
    }
    pub(crate) async fn write_to_partition(&self, partition_name: String, key: String, data: Vec<u8>) -> crate::Result<()> {
        // self.shared.state.
        // {
        //     let state = self.shared.state.read().await;
        //     if let Some(state) = state.partitions.get(&partition_name) {
        //         drop(state);
        //         return self._write_to_partition(partition_name, key, data).await;
        //     }
        // }
        //
        // self.open_partition(partition_name.clone()).await.unwrap();
        self._write_to_partition(partition_name, key, data).await?;
        Ok(())
    }

    async fn _write_to_partition(&self, partition_name: String, key: String, data: Vec<u8>) -> crate::Result<()> {
        let state = self.shared.state.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();
        partition.insert(&key, data)?;
        drop(state);
        Ok(())
    }

    pub(crate) async fn read_from_partition(&self, partition_name: String, key: String) -> crate::Result<Vec<u8>> {
        let state = self.shared.state.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();
        let value = partition.get(&key).unwrap().unwrap_or(Arc::new([]));
        drop(state);
        Ok(value.to_vec())
    }
    pub(crate) async fn read_last_from_partition(&self, partition_name: String, count: u16) -> crate::Result<Vec<Vec<u8>>> {
        let state = self.shared.state.read().await;
        let partition = state.partitions.get(&partition_name).unwrap();
        let values = partition.iter().into_iter().rev().take(count as usize)
            .map(|item| {
                let (k, v) = item.unwrap();
                info!("key: {:?}", k);
                v.to_vec()
            })
            .collect::<Vec<_>>();
        drop(state);
        Ok(
            values
        )
    }
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> Db {
        Db {
            shared: Arc::new(
            SharedState {
                state: RwLock::new(DbState {
                    keyspace: Config::new(path)
                        // .fsync_ms(Some(60_000))
                        .fsync_ms(Some(1000))
                        .block_cache(Arc::new(fjall::BlockCache::with_capacity_bytes(/* 16 MiB */ 64 * 1_024 * 1_024)))
                        .compaction_workers(1)
                        .open().unwrap(),
                    partitions: HashMap::new(),
                })
            })
        }
    }

    /*   pub fn open_partition(&mut self, name: &str) -> PartitionHandle {
           let partition = self.keyspace.open_partition(name, Default::default()).unwrap();
           self.partitions.insert(name.to_string(), partition);
           partition
       }

       pub fn get_partition(&self, name: &str) -> &PartitionHandle {
           self.partitions.get(name).unwrap()
       }*/
}
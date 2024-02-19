use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use axum::http::StatusCode;

use axum::Json;
use axum::response::IntoResponse;
use deadpool::managed::Object;
use fjall::{Batch, Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use lazy_static::lazy_static;
use rand::random;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use moonshine_db::client::Manager;
use moonshine_db::Pool;

use crate::model::{AccountBalance, AccountStatement, Transaction, TransactionDetail};

struct AppState {
    keyspace: Keyspace,
    db: PartitionHandle,
    db01: PartitionHandle,
    db02: PartitionHandle,
    db03: PartitionHandle,
    db04: PartitionHandle,
    db05: PartitionHandle,
}

impl AppState {
    pub fn db(&self, client_id: i32) -> &PartitionHandle {
        match client_id {
            1 => &self.db01,
            2 => &self.db02,
            3 => &self.db03,
            4 => &self.db04,
            5 => &self.db05,
            _ => panic!("Invalid client_id"),
        }
    }
}

lazy_static! {
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
/*    static ref APP_STATE: AppState = {
        let keyspace = Config::new(".db")
        .fsync_ms(Some(60_000))
        .block_cache(Arc::new(fjall::BlockCache::with_capacity_bytes(/* 16 MiB */ 64 * 1_024 * 1_024)))
        .open().unwrap();

        let config = fjall::PartitionCreateOptions::default().level_count(50);

        let db = keyspace
            .open_partition("my_items", config)
            .unwrap();
        db.set_compaction_strategy(Arc::new(fjall::compaction::Levelled {
                target_size: /* 512 KiB */ 64 * 1_024 * 1_024,
                l0_threshold: 2,
            }));
        db.set_max_memtable_size(64 * 1_024 * 1_024);

        let state = AppState {
            keyspace: keyspace.clone(),
            db: db.clone(),
            db01: keyspace
                .open_partition("my_items_01", PartitionCreateOptions::default().level_count(50))
                .unwrap(),
            db02: keyspace
                .open_partition("my_items_02", PartitionCreateOptions::default().level_count(50))
                .unwrap(),
            db03: keyspace
                .open_partition("my_items_03", PartitionCreateOptions::default().level_count(50))
                .unwrap(),
            db04: keyspace
                .open_partition("my_items_04", PartitionCreateOptions::default().level_count(50))
                .unwrap(),
            db05: keyspace
                .open_partition("my_items_05", PartitionCreateOptions::default().level_count(50))
                .unwrap()
        };

        state.db01.set_max_memtable_size(64 * 1_024 * 1_024);
        state.db02.set_max_memtable_size(64 * 1_024 * 1_024);
        state.db03.set_max_memtable_size(64 * 1_024 * 1_024);
        state.db04.set_max_memtable_size(64 * 1_024 * 1_024);
        state.db05.set_max_memtable_size(64 * 1_024 * 1_024);

        state
    };*/
    // static ref BINCODE_CONFIG: Configuration<dyn bincode::config::Config> =
    //     bincode::config::standard().with_big_endian();
}

#[derive(Deserialize, Serialize)]
struct Client {
    #[serde(skip)]
    id: i32,
    balance: i32,
    limit: i32,
}

impl Client {
    // pub fn store(&self, tree: &PartitionHandle) -> fjall::Result<()> {
    pub async fn store(&self, conn: &mut Object<Manager>) {
        let serialized = bincode::serialize(self).unwrap();
        conn.put("clients", &self.id.to_string(), serialized).await.unwrap();
        // tree.insert(&self.id.to_string(), serialized)?;
    }

    pub fn store_batch(&self, batch: &mut Batch, tree: &PartitionHandle) -> fjall::Result<()> {
        // let serialized = rmp_serde::to_vec(self).expect("should serialize");
        let serialized = bincode::serialize(self).unwrap();
        batch.insert(&tree, &self.id.to_string(), serialized);
        Ok(())
    }

    // pub fn load(tree: &PartitionHandle, key: &str) -> fjall::Result<Client> {
    pub async fn load(conn: &mut Object<Manager>, key: &str) -> fjall::Result<Client> {
        let item = conn.get("clients", key).await.unwrap();
        // let item = moonshine.cl.get("clients", key).await.unwrap();

        // let mut item: Client = rmp_serde::from_slice(&item).expect("should deserialize");
        let mut item: Client = bincode::deserialize(&item).unwrap();
        item.id = key.parse().unwrap();
        return Ok(item);
    }
}

impl TransactionDetail {
    pub fn store_batch(
        &self,
        batch: &mut Batch,
        tree: &PartitionHandle,
        client_id: i32,
    ) -> fjall::Result<()> {
        // let serialized = rmp_serde::to_vec(self).expect("should serialize");
        let serialized = bincode::serialize(self).unwrap();
        batch.insert(
            &tree,
            // format!("{}_{}", client_id, Uuid::new_v4().as_u128()),
            format!("{}_{}", client_id, random::<u32>().to_string()),
            serialized,
        );
        Ok(())
    }

    pub async fn store(&self, mut conn: Object<Manager>, client_id: i32) {
        let serialized = bincode::serialize(self).unwrap();
        conn.put(Self::get_partition(client_id),
                 &format!("{}_{}", client_id,
                          // COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                     Uuid::now_v7().as_u128()

                          // random::<u32>().to_string()
                 ),
                 serialized).await.unwrap();
        // tree.insert(&self.id.to_string(), serialized)?;
    }


    pub fn load(tree: &PartitionHandle, key: &str) -> fjall::Result<TransactionDetail> {
        let item = tree.get(key).unwrap().unwrap();

        // let item: TransactionDetail = rmp_serde::from_slice(&item).expect("should deserialize");
        let item: TransactionDetail = bincode::deserialize(&item).unwrap();
        return Ok(item);
    }

    pub fn get_partition(client_id: i32) -> &'static str {
        match client_id {
            1 => "my_items_01",
            2 => "my_items_02",
            3 => "my_items_03",
            4 => "my_items_04",
            5 => "my_items_05",
            _ => panic!("Invalid client_id"),
        }
    }
}

pub async fn init(conn: &mut Object<Manager>) {
    /**
    VALUES (1, 'Rocky Balboa', 100000),
           (2, 'Apollo Creed', 80000),
           (3, 'Mike Tyson', 1000000),
           (4, 'John Jones', 10000000),
           (5, 'Muhammad Ali', 500000);
    **/
    // if APP_STATE.db.len().unwrap() > 0 {
    //     return;
    // }

    conn.open_partition("clients").await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(1)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(2)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(3)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(4)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(5)).await.unwrap();

    // TODO: check if clients already exists
    // if conn.get("clients", "1").await.is_ok() {
    //     return;
    // }

    Client {
        id: 1,
        balance: 0,
        limit: 100000,
    }
    // .store(&APP_STATE.db)
    .store(conn).await;

    Client {
        id: 2,
        balance: 0,
        limit: 80000,
    }
    .store(conn).await;
    Client {
        id: 3,
        balance: 0,
        limit: 1000000,
    }
    .store(conn).await;
    Client {
        id: 4,
        balance: 0,
        limit: 10000000,
    }
    .store(conn).await;
    Client {
        id: 5,
        balance: 0,
        limit: 500000,
    }
    .store(conn).await;
}

pub(crate) async fn create_transaction(
    transaction_raw: Transaction,
    client_id: i32,
    value: i32,
    pool: Pool,
) -> Result<impl IntoResponse, StatusCode> {
    let transaction = TransactionDetail {
        valor: value,
        tipo: transaction_raw.tipo.clone().unwrap(),
        descricao: transaction_raw.descricao.clone().unwrap(),
        realizada_em: chrono::Utc::now().to_rfc3339(),
    };

    let mut conn = pool.get().await.unwrap();
    // let mut batch = APP_STATE.keyspace.batch();
    // let mut client = Client::load(&APP_STATE_STATE.db, &client_id.to_string()).unwrap();
    let mut client = Client::load(&mut conn, &client_id.to_string()).await.unwrap();
    let value_with_sign = if transaction.tipo.clone() == "d" {
        -value
    } else {
        value
    };

    client.balance += value_with_sign;
    let (balance, limit) = (client.balance, client.limit);
    if client.balance < client.limit * -1 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    client.store(&mut conn).await;

    // client.store_batch(&mut batch, &APP_STATE.db).unwrap();
    transaction
        // .store_batch(&mut batch, &APP_STATE.db(client_id), client_id)
        .store(conn, client_id)
        .await;



    // batch.commit().unwrap();

    return Ok(format!("{{\"limite\" : {limit}, \"saldo\" : {balance}}}"));
}

pub async fn get_account_statement(client_id: i32, pool: Pool) -> Json<AccountStatement> {
    // return Json(AccountStatement {
    //     saldo: AccountBalance {
    //         total: 0,
    //         data_extrato: "2024-02-16T20:25:14.185362+00:00".to_string(),
    //         limite: 100,
    //     },
    //     ultimas_transacoes: vec![],
    // });

    // let client = Client::load(&APP_STATE.db, &client_id.to_string()).unwrap();
    let mut conn = pool.get().await.unwrap();
    let client = Client::load(&mut conn, &client_id.to_string()).await.unwrap();
    let last_transactions = conn.get_last(TransactionDetail::get_partition(client_id), 10).await.unwrap()
        // APP_STATE
        // .db(client_id)
        .iter()
        // .into_iter()
        // .rev()
        // .take(10)
        .map(|item| {
            // let item = item.unwrap().1;
            // let item = item.unwrap().1;
            let transaction: TransactionDetail = bincode::deserialize(item).unwrap();
            transaction
        })
        .collect();

    return Json(AccountStatement {
        saldo: AccountBalance {
            total: client.balance,
            data_extrato: chrono::Utc::now().to_rfc3339(),
            limite: client.limit,
        },
        ultimas_transacoes: last_transactions,
    });
}

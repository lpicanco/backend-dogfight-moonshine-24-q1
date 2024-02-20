use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use chrono::{Datelike, Timelike};
use deadpool::managed::Object;
use fjall::PartitionHandle;
use lazy_static::lazy_static;
use rand::Rng;

use moonshine_db::client::Manager;
use moonshine_db::Pool;

use crate::model::{AccountBalance, AccountStatement, Transaction, TransactionDetail};

lazy_static! {
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
    static ref LIMITS: HashMap<i32, i32> = {
        let mut map = HashMap::new();
        map.insert(1, 100000);
        map.insert(2, 80000);
        map.insert(3, 1000000);
        map.insert(4, 10000000);
        map.insert(5, 500000);
        map
    };
}

impl TransactionDetail {
    pub async fn store(&self, mut conn: Object<Manager>, client_id: i32) {
        let serialized = bincode::serialize(self).unwrap();
        // println!("{}", Self::generate_id(client_id));
        conn.put(Self::get_partition(client_id),
                 &format!("{}_{}", client_id,
                     Self::generate_id(client_id)
                            // self.realizada_em
                          // Uuid::now_v7().as_u128()
                          // Uuid::now_v7().as_u64_pair().1
                 ),
                 serialized, true).await.unwrap();
    }

    fn generate_id(client_id: i32) -> String {
        let now = chrono::Utc::now();

        let day = now.day() as u8;
        let hour = now.hour() as u8;
        let min = now.minute() as u8;

        let sec = now.second() as u8;
        let nano = now.timestamp_subsec_nanos();

        // let mut rng = rand::thread_rng();
        // let random = rng.gen::<u16>();

        format!(
            "{}{}{}{}{}{}", //{:0>4}
            client_id as u8,
            day,
            hour,
            min,
            sec,
            nano)
            .into()
    }

    pub fn load(tree: &PartitionHandle, key: &str) -> fjall::Result<TransactionDetail> {
        let item = tree.get(key).unwrap().unwrap();
        let item: TransactionDetail = bincode::deserialize(&item).unwrap();
        return Ok(item);
    }

    pub fn get_partition(client_id: i32) -> &'static str {
        match client_id {
            1 => "tr01",
            2 => "tr02",
            3 => "tr03",
            4 => "tr04",
            5 => "tr05",
            _ => panic!("Invalid client_id"),
        }
    }
}

pub async fn init(conn: &mut Object<Manager>) {
    conn.open_partition("clients").await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(1)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(2)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(3)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(4)).await.unwrap();
    conn.open_partition(TransactionDetail::get_partition(5)).await.unwrap();
}

pub(crate) async fn create_transaction(
    transaction_raw: Transaction,
    client_id: i32,
    value: i32,
    pool: Pool,
) -> Result<impl IntoResponse, StatusCode> {
    let mut conn = pool.get().await.unwrap();

    let last_balance = match conn.get_latest(TransactionDetail::get_partition(client_id), true).await.unwrap() {
        Some(item) => {
            let detail: TransactionDetail = bincode::deserialize(&item).unwrap();
            detail.balance
        },
        None => 0,
    };

    let value_with_sign = if transaction_raw.tipo.clone().unwrap() == "d" {
        -value
    } else {
        value
    };

    let transaction = TransactionDetail {
        valor: value,
        tipo: transaction_raw.tipo.clone().unwrap(),
        descricao: transaction_raw.descricao.clone().unwrap(),
        realizada_em: chrono::Utc::now().to_rfc3339(),
        balance: last_balance + value_with_sign
    };

    let (balance, limit) = (transaction.balance, *LIMITS.get(&client_id).unwrap());
    if balance < limit * -1 {
        conn.unlock(TransactionDetail::get_partition(client_id)).await.unwrap();
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    transaction
        .store(conn, client_id)
        .await;

    return Ok(format!("{{\"limite\" : {limit}, \"saldo\" : {balance}}}"));
}

pub async fn get_account_statement(client_id: i32, pool: Pool) -> Json<AccountStatement> {
    let mut conn = pool.get().await.unwrap();
    let last_transactions: Vec<TransactionDetail> = conn.get_last(TransactionDetail::get_partition(client_id), 10).await.unwrap()
        .iter()
        .map(|item| {
            let transaction: TransactionDetail = bincode::deserialize(item).unwrap();
            transaction
        })
        .collect();

    let balance = match last_transactions.first() {
        Some(transaction) => transaction.balance,
        None => 0,
    };

    return Json(AccountStatement {
        saldo: AccountBalance {
            total: balance,
            data_extrato: chrono::Utc::now().to_rfc3339(),
            limite: *LIMITS.get(&client_id).unwrap(),
        },
        ultimas_transacoes: last_transactions,
    });
}

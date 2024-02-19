use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize)]
pub struct Transaction {
    #[validate(required, range(min = 1))]
    pub valor: Option<f32>,
    #[validate(required, length(min = 1, max = 1))]
    pub tipo: Option<String>,
    #[validate(required, length(min = 1, max = 10))]
    pub descricao: Option<String>,
}

#[derive(Serialize)]
pub struct AccountStatement {
    pub saldo: AccountBalance,
    pub ultimas_transacoes: Vec<TransactionDetail>,
}

#[derive(Serialize)]
pub struct AccountBalance {
    pub total: i32,
    pub data_extrato: String,
    pub limite: i32,
}

#[derive(Serialize, Deserialize)] //sqlx::FromRow,
pub struct TransactionDetail {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: String,
}

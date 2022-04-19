use disn::model::TxRecord;
use disn::CHAIN;

#[tokio::main]
async fn main() {
  let mut records = TxRecord::find_by_send_status(0, CHAIN.pool.clone())
      .await
      .unwrap()
      .into_iter();
  while let Some(record) = records.next() {
      CHAIN.confirm_tx(record.tx_hash).await;
  }

}
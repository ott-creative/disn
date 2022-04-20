use disn::model::TxRecord;
use disn::CHAIN;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    let mut records = TxRecord::find_by_send_status(0).await.unwrap().into_iter();
    let mut tasks: Vec<JoinHandle<()>> = vec![];
    while let Some(record) = records.next() {
        let tx_hash = record.tx_hash;
        let task = tokio::spawn(async move {
            CHAIN.confirm_tx(tx_hash).await;
        });
        tasks.push(task);
    }
    let mut tasks_iter = tasks.into_iter();
    while let Some(task) = tasks_iter.next() {
        let _ = tokio::join!(task);
    }
}

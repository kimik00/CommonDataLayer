#![feature(iterator_fold_self)]

use anyhow::Context;
use postgres::{Client, NoTls, Row};
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, Clone, StructOpt)]
struct Opts {
    #[structopt(env = "POSTGRESQL_CONNECTION")]
    pub connection_string: String,
}

/// This app is called by some external scheduler (eg. cron),
/// It merges all documents with same object_id into one.
fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    let mut client =
        Client::connect(&opts.connection_string, NoTls).context("Connection to PostgreSQL")?;

    let changed_ids = client
        .query(
            "SELECT object_id FROM data GROUP BY object_id HAVING count(*) > 1",
            &[],
        )
        .context("Select duplicated object_ids")?;

    for id in changed_ids {
        let object_id: Uuid = id.get(0);
        shrink_by_id(&mut client, object_id)
            .with_context(|| format!("Shrinking of {}", object_id))?;
    }

    Ok(())
}

fn shrink_by_id(client: &mut Client, object_id: Uuid) -> anyhow::Result<()> {
    let rows = get_all_with_id(client, object_id)?;
    let version: i64 = rows
        .last()
        .context("No rows with selected ID, did database change mid-shrink?")?
        .get(0);

    let document = minimize_document(rows).context("Failed minimizing document")?;

    let mut transaction = client.transaction()?;

    transaction.query(
        "DELETE FROM data WHERE object_id = $1 AND version < $2",
        &[&object_id, &version],
    )?;
    transaction.query(
        "UPDATE data SET payload = $1 WHERE object_id = $2 AND version = $3",
        &[&document, &object_id, &version],
    )?;

    transaction.commit()?;

    Ok(())
}

fn minimize_document(rows: impl IntoIterator<Item = Row>) -> Option<JsonValue> {
    rows.into_iter()
        .map(|row| row.get::<_, JsonValue>(1))
        .fold_first(|mut acc, next| {
            merge(&mut acc, next);
            acc
        })
}

fn get_all_with_id(client: &mut Client, object_id: Uuid) -> anyhow::Result<Vec<Row>> {
    let entries = client.query(
        "SELECT version, payload FROM data WHERE object_id = $1 ORDER BY version ASC",
        &[&object_id],
    )?;
    Ok(entries)
}

fn merge(a: &mut JsonValue, b: JsonValue) {
    match (a, b) {
        (JsonValue::Object(ref mut a), JsonValue::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(JsonValue::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

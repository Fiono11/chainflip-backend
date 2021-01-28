use std::u64;

use super::{ILocalStore, LocalEvent, StorageItem};
use chainflip_common::types::chain::Witness;
use rusqlite::Connection as DB;
use rusqlite::{params, NO_PARAMS};

/// Implementation of ILocalStore that uses sqlite to
/// persist between restarts
pub struct PersistentLocalStore {
    db: DB,
}

fn create_tables_if_new(db: &DB) {
    db.execute(
        "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                data BLOB NOT NULL
    )",
        NO_PARAMS,
    )
    .expect("could not create or open DB");
}

impl PersistentLocalStore {
    /// Create a instance of PersistentLocalStore associated with a database file
    /// with name `file`. The file is created if does not exist. The database tables
    /// are created they don't already exist.
    pub fn open(file: &str) -> Self {
        let db = DB::open(file).expect("Could not open the database");

        create_tables_if_new(&db);

        // Load the events into memory here

        PersistentLocalStore { db }
    }
}

impl ILocalStore for PersistentLocalStore {
    fn add_events(&mut self, events: Vec<LocalEvent>) -> Result<(), String> {
        for event in events {
            let id = event.unique_id();
            let blob = serde_json::to_string(&event).unwrap();
            match self.db.execute(
                "
            INSERT INTO events
            (id, data) VALUES (?1, ?2)
            ",
                params![id, blob],
            ) {
                Ok(_) => {
                    trace!("Event ({:#} added to db", id);
                }
                Err(e) => {
                    return Err(format!("Event {:#} could not be added to db, {:#?}", id, e));
                }
            }
        }
        Ok(())
    }

    fn get_events(&mut self, last_seen: u64) -> Option<Vec<LocalEvent>> {
        let mut select_events = self
            .db
            .prepare("SELECT data FROM events WHERE rowid > ?")
            .expect("Could not prepare stmt");

        let mut rows = select_events
            // only u32 or smaller is castable to a SQL type
            .query(params![last_seen as u32])
            .map_err(|err| err.to_string())
            .unwrap();

        let mut recent_events: Vec<LocalEvent> = Vec::new();

        while let Some(row) = rows.next().ok()? {
            let str_val: String = row.get(0).unwrap();
            let l_evt = serde_json::from_str::<LocalEvent>(&str_val).unwrap();
            recent_events.push(l_evt);
        }

        Some(recent_events)
    }

    fn total_events(&mut self) -> u64 {
        let mut total_events = self
            .db
            .prepare("SELECT COUNT(*) FROM events")
            .expect("Could not prepare stmt");

        let count: Result<u32, _> = total_events.query_row(NO_PARAMS, |row| row.get(0));

        count.unwrap() as u64
    }

    fn get_witnesses(&mut self, last_event: u64) -> Option<Vec<Witness>> {
        let events = self.get_events(last_event);
        let mut witnesses: Vec<Witness> = vec![];
        for event in events? {
            if let LocalEvent::Witness(w) = event {
                witnesses.push(w);
            } else {
                // skip
            }
        }
        Some(witnesses)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::test_utils::{self, data::TestData};
    use chainflip_common::types::{coin::Coin, UUIDv4};

    #[test]
    fn test_db_created_successfully() {
        let temp_file = test_utils::TempRandomFile::new();

        let mut db = PersistentLocalStore::open(temp_file.path());

        let evt: LocalEvent = TestData::witness(UUIDv4::new(), 100, Coin::ETH).into();

        db.add_events(vec![evt.clone()])
            .expect("Error adding an event to the database");

        // Close the database
        drop(db);

        let mut db = PersistentLocalStore::open(temp_file.path());

        let events = db.get_events(0).unwrap();
        assert_eq!(events.len(), 1);
        let first_evt = events.first().unwrap();

        if let LocalEvent::Witness(w) = first_evt {
            assert_eq!(w.amount, 100);
        };
    }

    #[test]
    fn get_all_events() {
        let temp_file = test_utils::TempRandomFile::new();

        let mut db = PersistentLocalStore::open(temp_file.path());

        let evt: LocalEvent = TestData::witness(UUIDv4::new(), 100, Coin::ETH).into();
        let evt2: LocalEvent = LocalEvent::DepositQuote(TestData::deposit_quote(Coin::ETH));

        db.add_events(vec![evt.clone(), evt2.clone()])
            .expect("Error adding an event to the database");

        let all_events = db.get_events(0).unwrap();

        assert_eq!(all_events.len(), 2);
    }

    #[test]
    fn get_events_last_seen_non_zero() {
        let temp_file = test_utils::TempRandomFile::new();

        let mut db = PersistentLocalStore::open(temp_file.path());

        let evt: LocalEvent = TestData::witness(UUIDv4::new(), 100, Coin::ETH).into();
        let evt2: LocalEvent = LocalEvent::DepositQuote(TestData::deposit_quote(Coin::ETH));

        db.add_events(vec![evt.clone(), evt2.clone()])
            .expect("Error adding an event to the database");

        let all_events = db.get_events(1).unwrap();

        assert_eq!(all_events.len(), 1);
    }

    #[test]
    fn get_total_events() {
        let temp_file = test_utils::TempRandomFile::new();

        let mut db = PersistentLocalStore::open(temp_file.path());

        let evt: LocalEvent = TestData::witness(UUIDv4::new(), 100, Coin::ETH).into();
        let evt2: LocalEvent = LocalEvent::DepositQuote(TestData::deposit_quote(Coin::ETH));

        assert_eq!(db.total_events(), 0);

        db.add_events(vec![evt.clone(), evt2.clone()])
            .expect("Error adding an event to the database");

        assert_eq!(db.total_events(), 2);
    }
}

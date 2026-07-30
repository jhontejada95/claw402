//! Persistent, fail-closed daily spending ledger.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("daily spending cap must be greater than zero")]
    Disabled,
    #[error("amount must be greater than zero")]
    InvalidAmount,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemClock,
    #[error("budget storage failed: {0}")]
    Storage(String),
    #[error("purchase already has a budget record")]
    DuplicatePurchase,
    #[error("daily spending cap exceeded: reserved {reserved}, requested {requested}, cap {cap}")]
    CapExceeded {
        reserved: u64,
        requested: u64,
        cap: u64,
    },
    #[error("budget reservation does not exist or is no longer pending")]
    InvalidReservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetSnapshot {
    pub utc_day: u64,
    pub cap_atomic: u64,
    pub reserved_atomic: u64,
    pub remaining_atomic: u64,
}

pub struct BudgetLedger {
    connection: Connection,
    daily_cap_atomic: u64,
}

impl BudgetLedger {
    pub fn open(path: &Path, daily_cap_atomic: u64) -> Result<Self, BudgetError> {
        if daily_cap_atomic == 0 {
            return Err(BudgetError::Disabled);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(storage)?;
        }

        let connection = Connection::open(path).map_err(storage)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(storage)?;
        connection
            .execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = FULL;
                CREATE TABLE IF NOT EXISTS budget_entries (
                    purchase_id TEXT PRIMARY KEY,
                    utc_day INTEGER NOT NULL,
                    amount_atomic INTEGER NOT NULL CHECK (amount_atomic > 0),
                    status TEXT NOT NULL CHECK (status IN ('pending', 'settled', 'released')),
                    transaction_signature TEXT,
                    created_at_unix_seconds INTEGER NOT NULL,
                    updated_at_unix_seconds INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS budget_entries_day_status
                    ON budget_entries (utc_day, status);
                ",
            )
            .map_err(storage)?;

        Ok(Self {
            connection,
            daily_cap_atomic,
        })
    }

    /// Atomically reserves daily capacity before signing.
    pub fn reserve(
        &mut self,
        purchase_id: &str,
        amount_atomic: u64,
    ) -> Result<BudgetSnapshot, BudgetError> {
        if amount_atomic == 0 {
            return Err(BudgetError::InvalidAmount);
        }
        let now = unix_seconds()?;
        let day = now / 86_400;
        let amount_i64 = to_i64(amount_atomic)?;
        let cap = self.daily_cap_atomic;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;

        let existing: Option<String> = transaction
            .query_row(
                "SELECT status FROM budget_entries WHERE purchase_id = ?1",
                params![purchase_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?;
        if existing.is_some() {
            return Err(BudgetError::DuplicatePurchase);
        }

        let reserved_i64: i64 = transaction
            .query_row(
                "
                SELECT COALESCE(SUM(amount_atomic), 0)
                FROM budget_entries
                WHERE utc_day = ?1 AND status IN ('pending', 'settled')
                ",
                params![to_i64(day)?],
                |row| row.get(0),
            )
            .map_err(storage)?;
        let reserved = u64::try_from(reserved_i64).map_err(storage)?;
        let projected = reserved
            .checked_add(amount_atomic)
            .ok_or(BudgetError::CapExceeded {
                reserved,
                requested: amount_atomic,
                cap,
            })?;
        if projected > cap {
            return Err(BudgetError::CapExceeded {
                reserved,
                requested: amount_atomic,
                cap,
            });
        }

        transaction
            .execute(
                "
                INSERT INTO budget_entries (
                    purchase_id, utc_day, amount_atomic, status,
                    created_at_unix_seconds, updated_at_unix_seconds
                ) VALUES (?1, ?2, ?3, 'pending', ?4, ?4)
                ",
                params![purchase_id, to_i64(day)?, amount_i64, to_i64(now)?],
            )
            .map_err(storage)?;
        transaction.commit().map_err(storage)?;

        Ok(BudgetSnapshot {
            utc_day: day,
            cap_atomic: cap,
            reserved_atomic: projected,
            remaining_atomic: cap - projected,
        })
    }

    pub fn settle(
        &mut self,
        purchase_id: &str,
        transaction_signature: &str,
    ) -> Result<(), BudgetError> {
        if transaction_signature.is_empty() {
            return Err(BudgetError::InvalidReservation);
        }
        let updated = self
            .connection
            .execute(
                "
                UPDATE budget_entries
                SET status = 'settled', transaction_signature = ?2,
                    updated_at_unix_seconds = ?3
                WHERE purchase_id = ?1 AND status = 'pending'
                ",
                params![purchase_id, transaction_signature, to_i64(unix_seconds()?)?],
            )
            .map_err(storage)?;
        if updated != 1 {
            return Err(BudgetError::InvalidReservation);
        }
        Ok(())
    }

    pub fn release(&mut self, purchase_id: &str) -> Result<(), BudgetError> {
        let updated = self
            .connection
            .execute(
                "
                UPDATE budget_entries
                SET status = 'released', updated_at_unix_seconds = ?2
                WHERE purchase_id = ?1 AND status = 'pending'
                ",
                params![purchase_id, to_i64(unix_seconds()?)?],
            )
            .map_err(storage)?;
        if updated != 1 {
            return Err(BudgetError::InvalidReservation);
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<BudgetSnapshot, BudgetError> {
        let day = unix_seconds()? / 86_400;
        let reserved_i64: i64 = self
            .connection
            .query_row(
                "
                SELECT COALESCE(SUM(amount_atomic), 0)
                FROM budget_entries
                WHERE utc_day = ?1 AND status IN ('pending', 'settled')
                ",
                params![to_i64(day)?],
                |row| row.get(0),
            )
            .map_err(storage)?;
        let reserved = u64::try_from(reserved_i64).map_err(storage)?;
        Ok(BudgetSnapshot {
            utc_day: day,
            cap_atomic: self.daily_cap_atomic,
            reserved_atomic: reserved,
            remaining_atomic: self.daily_cap_atomic.saturating_sub(reserved),
        })
    }
}

fn unix_seconds() -> Result<u64, BudgetError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BudgetError::InvalidSystemClock)
        .map(|duration| duration.as_secs())
}

fn to_i64(value: u64) -> Result<i64, BudgetError> {
    i64::try_from(value).map_err(storage)
}

fn storage(error: impl std::fmt::Display) -> BudgetError {
    BudgetError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn reserves_settles_and_persists_daily_spend() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("budget.sqlite");
        {
            let mut ledger = BudgetLedger::open(&path, 10_000).unwrap();
            let reserved = ledger.reserve("purchase-a", 3_000).unwrap();
            assert_eq!(reserved.remaining_atomic, 7_000);
            ledger.settle("purchase-a", "devnet-signature").unwrap();
        }

        let ledger = BudgetLedger::open(&path, 10_000).unwrap();
        let snapshot = ledger.snapshot().unwrap();
        assert_eq!(snapshot.reserved_atomic, 3_000);
        assert_eq!(snapshot.remaining_atomic, 7_000);
    }

    #[test]
    fn pending_reservations_count_toward_the_cap() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("budget.sqlite");
        let mut ledger = BudgetLedger::open(&path, 10_000).unwrap();
        ledger.reserve("purchase-a", 7_000).unwrap();

        assert!(matches!(
            ledger.reserve("purchase-b", 3_001),
            Err(BudgetError::CapExceeded {
                reserved: 7_000,
                requested: 3_001,
                cap: 10_000
            })
        ));
    }

    #[test]
    fn released_reservations_restore_capacity() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("budget.sqlite");
        let mut ledger = BudgetLedger::open(&path, 10_000).unwrap();
        ledger.reserve("purchase-a", 8_000).unwrap();
        ledger.release("purchase-a").unwrap();

        let snapshot = ledger.snapshot().unwrap();
        assert_eq!(snapshot.reserved_atomic, 0);
        assert_eq!(snapshot.remaining_atomic, 10_000);
    }

    #[test]
    fn duplicate_purchase_ids_fail_closed() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("budget.sqlite");
        let mut ledger = BudgetLedger::open(&path, 10_000).unwrap();
        ledger.reserve("purchase-a", 1_000).unwrap();

        assert!(matches!(
            ledger.reserve("purchase-a", 1_000),
            Err(BudgetError::DuplicatePurchase)
        ));
    }
}

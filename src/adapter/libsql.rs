// Copyright © 2024 Aris Ripandi - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
 * Portions of this file are based on code from `chrisllontop/keyv-rust`.
 * MIT Licensed, Copyright (c) 2023 Christian Llontop.
 *
 * Credits to Alexandru Bereghici: https://github.com/chrisllontop/keyv-rust
 */

use libsql::{params, params_from_iter};
use libsql::{Builder, Connection};
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::{Store, StoreError, StoreModel, DEFAULT_NAMESPACE_NAME};

/// Builder for creating a `KyvalStore`.
///
/// This builder allows for configuring a `KyvalStore` with custom
/// settings such as a specific database file URI and a table name.
/// It provides a flexible way to initialize the store depending on the
/// application's requirements.
///
/// # Examples
///
/// ## Initializing with a Database File URI
///
/// ```rust,no_run
/// # use kyval::{KyvalStoreBuilder};
///
/// # #[tokio::main]
/// # async fn main(){
/// let store = KyvalStoreBuilder::new()
///     .uri(":memory:")
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
///  }
/// ```
///
/// ## Using an Existing Connection Pool
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use kyval::{KyvalStoreBuilder};
/// # use libsql::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let conn: Arc<Connnection> = Arc::new(Connection::open(path).unwrap());
///
/// let store = KyvalStoreBuilder::new()
///     .connnection(conn)
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
/// }
/// ```
pub struct KyvalStoreBuilder {
    uri: Option<PathBuf>,
    token: Option<String>,
    connnection: Option<Arc<Connection>>,
    table_name: Option<String>,
}

impl KyvalStoreBuilder {
    pub fn new() -> Self {
        Self {
            uri: None,
            token: None,
            connnection: None,
            table_name: None,
        }
    }

    /// Sets the table name for the `KyvalStore`.
    ///
    /// This method configures the table name to be used by the store. If not set,
    /// `DEFAULT_TABLE_NAME` from the configuration will be used.
    pub fn table_name<S: Into<String>>(mut self, table: S) -> Self {
        self.table_name = Some(table.into());
        self
    }

    /// Sets the database URI for connecting to the SQLite database.
    ///
    /// This method configures the database URI. It's required if no existing connection is provided.
    pub fn uri<S: Into<PathBuf>>(mut self, uri: S) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets the database token for authentication with the database.
    ///
    /// This method configures the database token. It's required if using authentication.
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Uses an existing connection for the `KyvalStore`.
    ///
    /// This method allows for using an already configured `Pool`. If set,
    /// the `uri` option is ignored.
    pub fn connnection(mut self, connnection: Arc<Connection>) -> Self {
        self.connnection = Some(connnection);
        self
    }

    /// Builds the `KyvalStore` based on the provided configurations.
    ///
    /// Finalizes the builder and creates an `KyvalStore` instance.
    /// It requires either a database URI or an existing connection to be set.
    ///
    /// # Returns
    /// This method returns a `Result` which, on success, contains the initialized `KyvalStore`.
    /// On failure, it returns a `StoreError` indicating what went wrong during the initialization.
    pub async fn build(self) -> Result<KyvalStore, StoreError> {
        let connnection = match self.connnection {
            Some(connnection) => connnection,
            None => {
                let path = self
                    .uri
                    .expect("KyvalStore requires either a URI or an existing connnection to be set");

                // If the token is set, use the remote database connection.
                let db = if let Some(token) = self.token {
                    Builder::new_remote(path.display().to_string(), token)
                        .build()
                        .await
                        .map_err(|_| {
                            StoreError::ConnectionError(
                                "Failed to create database connection"
                                    .to_string(),
                            )
                        })?
                } else {
                    Builder::new_local(path).build().await.map_err(|_| {
                        StoreError::ConnectionError(
                            "Failed to create database connection".to_string(),
                        )
                    })?
                };

                let conn = db.connect().map_err(|_| {
                    StoreError::ConnectionError(
                        "Failed to create database connnection".to_string(),
                    )
                })?;

                Arc::new(conn)
            }
        };

        let table_name = self.table_name.unwrap_or_else(|| {
            log::warn!("Table name not set, using default table name");
            DEFAULT_NAMESPACE_NAME.to_string()
        });

        Ok(KyvalStore {
            connnection,
            table_name,
        })
    }
}

pub struct KyvalStore {
    pub(crate) connnection: Arc<Connection>,
    pub(crate) table_name: String,
}

impl KyvalStore {
    fn get_table_name(&self) -> String {
        self.table_name.clone()
    }
}

impl Store for KyvalStore {
    fn initialize(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {table_name} (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TEXT DEFAULT (datetime('now', 'localtime')),
                    UNIQUE(key)
                ) STRICT;
                CREATE INDEX IF NOT EXISTS {table_name}_key_idx ON {table_name} (key);
                CREATE TRIGGER IF NOT EXISTS {table_name}_update_trigger
                AFTER UPDATE ON {table_name}
                BEGIN
                    UPDATE {table_name} SET updated_at = datetime('now', 'localtime') WHERE key = NEW.key;
                END;
            "#,
            table_name = self.get_table_name()
        );

        let conn = &*self.connnection;

        Box::pin(async move {
            conn.execute_batch(&query).await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to initialize the database table: {}",
                    e
                ))
            })?;

            Ok(())
        })
    }

    fn get(
        &self,
        key: &str,
    ) -> Pin<
        Box<dyn Future<Output = Result<Option<Value>, StoreError>> + Send + '_>,
    > {
        let query = format!(
            "SELECT value FROM {} WHERE key = ?1 LIMIT 1",
            self.get_table_name()
        );

        let conn = &*self.connnection;
        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn.prepare(&query).await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to set the statement: {:?}",
                    e
                ))
            })?;

            let result =
                stmt.query_row(params![key.clone()]).await.map_err(|e| {
                    StoreError::QueryError(format!(
                        "Failed to fetch the value: {:?}",
                        e
                    ))
                })?;

            let row_value: String = result.get(0).map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to get the value: {:?}",
                    e
                ))
            })?;

            let value = serde_json::to_value(row_value)
                .map_err(|e| StoreError::SerializationError { source: e })?;

            let duration = start.elapsed();
            log::debug!(
                "Kyval store get: {:?} | {} | {:?}",
                duration,
                key,
                value
            );

            Ok(Some(value))
        })
    }

    fn list(
        &self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<StoreModel>, StoreError>>
                + Send
                + '_,
        >,
    > {
        let query = format!(
            "SELECT key, value FROM {} ORDER BY key ASC;",
            self.get_table_name()
        );

        let conn = &*self.connnection;

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn.prepare(&query).await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to set the statement: {:?}",
                    e
                ))
            })?;

            let mut results = stmt.query(params![]).await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to fetch the value: {:?}",
                    e
                ))
            })?;

            let mut items: Vec<StoreModel> = Vec::new();

            while let Some(row) = results.next().await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to iterate rows: {:?}",
                    e
                ))
            })? {
                let key: String = row.get(0).map_err(|e| {
                    StoreError::QueryError(format!(
                        "Failed to get the value: {:?}",
                        e
                    ))
                })?;
                let row_value: String = row.get(1).map_err(|e| {
                    StoreError::QueryError(format!(
                        "Failed to get the value: {:?}",
                        e
                    ))
                })?;
                let value = serde_json::to_value(row_value).map_err(|e| {
                    StoreError::SerializationError { source: e }
                })?;

                items.push(StoreModel { key, value });
            }

            let duration = start.elapsed();
            log::debug!("Kyval store list: {:?} | {:?}", duration, items);

            Ok(items)
        })
    }

    fn set(
        &self,
        key: &str,
        value: Value,
        _ttl: Option<u64>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<StoreModel>, StoreError>>
                + Send
                + '_,
        >,
    > {
        let query = format!(
            "INSERT INTO {} (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = EXCLUDED.value",
            self.get_table_name()
        );

        let conn = &*self.connnection;
        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let value_str = match value {
                Value::String(ref s) => s.clone(), // If the value is a string, use the original string.
                Value::Number(ref n) => n.to_string(), // If the value is a number, use the number string representation.
                Value::Null => "".to_string(), // If value is null, use the empty string.
                _ => value.to_string(), // If the value is an object or other type, serialize it as JSON.
            };

            let mut stmt = conn.prepare(&query).await.map_err(|_| {
                StoreError::QueryError(
                    "Failed to set the statement".to_string(),
                )
            })?;

            let mut response = stmt
                .query(params![key.clone(), value_str.clone()])
                .await
                .map_err(|_| {
                    StoreError::QueryError(
                        "Failed to set the value".to_string(),
                    )
                })?;

            let result = match response.next().await.map_err(|e| {
                StoreError::QueryError(format!(
                    "Failed to iterate rows: {:?}",
                    e
                ))
            })? {
                Some(row) => {
                    let row_key = row.get_str(0).map_err(|e| {
                        StoreError::QueryError(format!(
                            "Failed to get the key: {:?}",
                            e
                        ))
                    })?;

                    let row_value = row.get_value(1).map_err(|e| {
                        StoreError::QueryError(format!(
                            "Failed to get the value: {:?}",
                            e
                        ))
                    })?;

                    Some(StoreModel {
                        key: row_key.to_string(),
                        value: serde_json::to_value(row_value).map_err(
                            |e| StoreError::SerializationError { source: e },
                        )?,
                    })
                }
                None => None,
            };

            let duration = start.elapsed();
            log::debug!(
                "Kyval store set: {:?} | {} | {}",
                duration,
                key,
                value_str
            );

            Ok(result)
        })
    }

    fn remove(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query =
            format!("DELETE FROM {} WHERE key = ?1", self.get_table_name());

        let conn = &*self.connnection;

        let key = key.to_string();

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn.prepare(&query).await.map_err(|_| {
                StoreError::QueryError(
                    "Failed to set the statement".to_string(),
                )
            })?;

            stmt.execute(params![key.clone()]).await.map_err(|_| {
                StoreError::QueryError("Failed to remove the key".to_string())
            })?;

            let duration = start.elapsed();
            log::debug!("Kyval store remove: {:?} | {}", duration, key);

            Ok(())
        })
    }

    fn remove_many(
        &self,
        keys: &[&str],
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let conn = &*self.connnection;

        let placeholder = keys
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<String>>()
            .join(", ");

        let query = format!(
            "DELETE FROM {} WHERE key IN ({})",
            self.get_table_name(),
            placeholder
        );

        let keys = keys.iter().map(|k| k.to_string()).collect::<Vec<String>>();

        Box::pin(async move {
            let start = Instant::now();

            let mut stmt = conn.prepare(&query).await.map_err(|_| {
                StoreError::QueryError(
                    "Failed to set the statement".to_string(),
                )
            })?;

            stmt.execute(params_from_iter(keys)).await.map_err(|_| {
                StoreError::QueryError("Failed to remove the key".to_string())
            })?;

            let duration = start.elapsed();
            log::debug!("Kyval store remove_many: {:?}", duration);

            Ok(())
        })
    }

    fn clear(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>> {
        let query = format!("DELETE FROM {}", self.get_table_name());

        let conn = &*self.connnection;

        Box::pin(async move {
            conn.execute(&query, params![]).await.map_err(|_| {
                StoreError::QueryError("Failed to clear the table".to_string())
            })?;

            Ok(())
        })
    }
}

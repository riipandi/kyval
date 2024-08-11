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

use serde::Serialize;
use serde_json::Value;
use std::{path::Path, sync::Arc};

use crate::adapter::KyvalStoreBuilder;
use crate::{Store, StoreError, StoreModel};

#[derive(thiserror::Error, Debug)]
pub enum KyvalError {
    #[error("Store error: {0}")]
    StoreError(#[from] StoreError),
}

/// Key-Value Store Interface
///
/// Provides an synchronous interface to a key-value store. This implementation
/// allows for setting, getting, removing, and clearing key-value pairs in a
/// datastore with an optional Time-to-Live (TTL) for keys.
///
/// The `Kyval` struct is generic over any implementation of the `Store` trait,
/// thus can be backed by various storage engines.
///
/// # Examples
///
/// ## Create a new instance with in-memory store
///
/// ```
/// # use kyval::Kyval;
/// let kyval = Kyval::default();
/// ```
///
/// ## Set and get a value
///
/// ```rust,no_run
/// # use kyval::Kyval;
/// #[tokio::main]
/// async fn main() {
///     let kyval = Kyval::default();
///
///     kyval.set("array", vec!["hola", "test"]).await.unwrap();
///
///     match kyval.get("array").await.unwrap() {
///         Some(array) => {
///             let array: Vec<String> = serde_json::from_value(array).unwrap();
///             assert_eq!(array, vec!["hola".to_string(), "test".to_string()])
///         }
///         None => assert!(false),
///     }
///
///     kyval.set("string", "life long").await.unwrap();
///     match kyval.get("string").await.unwrap() {
///         Some(string) => {
///             let string: String = serde_json::from_value(string).unwrap();
///             assert_eq!(string, "life long");
///         }
///         None => assert!(false),
///     }
/// }
/// ```
pub struct Kyval {
    store: Arc<dyn Store>,
}

impl Kyval {
    /// Attempts to create a new `Kyval` instance with a custom store.
    ///
    /// This function will attempt to initialize the provided store. If the initialization
    /// is successful, a new `Kyval` instance is returned.
    ///
    /// # Arguments
    ///
    /// * `store` - A custom store implementing the `Store` trait.
    ///
    /// # Errors
    ///
    /// Returns `KyvalError` if the store fails to initialize.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::{Kyval};
    /// # use kyval::adapter::KyvalStoreBuilder;
    /// #[tokio::main]
    /// async fn main() {
    /// let store = KyvalStoreBuilder::new()
    ///     .uri(":memory:")
    ///     .table_name("custom_table_name")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let kyval = Kyval::try_new(store).await.unwrap();
    /// }
    /// ```
    pub async fn try_new<S: Store + 'static>(
        store: S,
    ) -> Result<Self, KyvalError> {
        store.initialize().await?;
        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Sets a value for a given key without a TTL.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which the value is stored.
    /// * `value` - The value to store. Must implement `Serialize`.
    ///
    /// # Errors
    ///
    /// Returns `KyvalError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///     kyval.set("key", "hello world").await.unwrap();
    /// }
    /// ```
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: T,
    ) -> Result<Option<StoreModel>, KyvalError> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| StoreError::SerializationError { source: e })?;
        Ok(self.store.set(key, json_value, None).await?)
    }

    /// Sets a value for a given key with an expiry TTL (Time-To-Live).
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    /// * `value` - The value to be stored, which must implement `Serialize`.
    /// * `ttl` - The time-to-live (in seconds) for the key-value pair.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result on successful insertion, or a `KyvalError` on failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///     kyval.set_with_ttl("temp_key", "temp_value", 3600).await.unwrap(); // Expires in 1 hour
    /// }
    /// ```
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: T,
        ttl: u64,
    ) -> Result<Option<StoreModel>, KyvalError> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| StoreError::SerializationError { source: e })?;
        Ok(self.store.set(key, json_value, Some(ttl)).await?)
    }

    /// Retrieves a value based on a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result with `Option<Value>` on success, where `None` indicates the
    /// key does not exist, or a `KyvalError` on failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///
    ///     kyval.set("array", vec!["hola", "test"]).await.unwrap();
    ///
    ///     match kyval.get("array").await.unwrap() {
    ///         Some(array) => {
    ///             let array: Vec<String> = serde_json::from_value(array).unwrap();
    ///             assert_eq!(array, vec!["hola".to_string(), "test".to_string()])
    ///         }
    ///         None => assert!(false),
    ///     }
    ///
    ///     kyval.set("string", "life long").await.unwrap();
    ///     match kyval.get("string").await.unwrap() {
    ///         Some(string) => {
    ///             let string: String = serde_json::from_value(string).unwrap();
    ///             assert_eq!(string, "life long");
    ///         }
    ///         None => assert!(false),
    ///     }
    /// }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<Value>, KyvalError> {
        Ok(self.store.get(key).await?)
    }

    /// Lists all key-value pairs stored in the Kyval store.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Vec` of tuples, where each tuple contains the key (as a `String`) and the corresponding value (as a `Value`). If an error occurs, a `KyvalError` is returned.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///
    ///     let pairs = kyval.list().await.unwrap();
    ///
    ///     for item in pairs {
    ///         println!("Key: {}, Value: {}", item.key, item.value);
    ///     }
    /// }
    /// ```
    pub async fn list(&self) -> Result<Vec<StoreModel>, KyvalError> {
        Ok(self.store.list().await?)
    }

    /// Removes a specified key from the store.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that represents the key to be removed.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the key has been successfully removed, or a `KyvalError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///     kyval.remove("my_key").await.unwrap(); // Removes "my_key" from the store
    /// }
    /// ```
    pub async fn remove(&self, key: &str) -> Result<(), KyvalError> {
        Ok(self.store.remove(key).await?)
    }

    /// Removes multiple keys from the store in one operation.
    ///
    /// # Arguments
    ///
    /// * `keys` - A slice of strings or string-like objects that represent the keys to be removed.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the keys have been successfully removed, or a `KyvalError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///     kyval.remove_many(&["key1", "key2"]).await.unwrap(); // Removes "key1" and "key2"
    /// }
    /// ```
    pub async fn remove_many<T: AsRef<str> + Sync>(
        &self,
        keys: &[T],
    ) -> Result<(), KyvalError> {
        let keys: Vec<&str> = keys.iter().map(|k| k.as_ref()).collect();
        Ok(self.store.remove_many(&keys).await?)
    }

    /// Clears the entire store, removing all key-value pairs.
    ///
    /// # Returns
    ///
    /// Returns an `Ok` result if the store has been successfully cleared, or a `KyvalError`
    /// on failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kyval::Kyval;
    /// #[tokio::main]
    /// async fn main() {
    ///     let kyval = Kyval::default();
    ///     kyval.clear().await.unwrap(); // Clears the entire store
    /// }
    /// ```
    pub async fn clear(&self) -> Result<(), KyvalError> {
        Ok(self.store.clear().await?)
    }
}

/// Provides a default implementation for the `Kyval` struct, which creates an in-memory store.
/// This is useful for quickly setting up a `Kyval` instance without needing to configure a
/// specific storage backend.
impl Default for Kyval {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create async runtime");
        let store = runtime.block_on(async {
            KyvalStoreBuilder::new()
                .uri(Path::new(":memory:"))
                .build()
                .await
                .expect("Failed to build KyvalStore")
        });
        Self {
            store: Arc::new(store),
        }
    }
}

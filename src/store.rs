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

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreModel {
    pub key: String,
    pub value: Value,
}

pub trait Store: Send + Sync {
    /// Initializes the storage backend.
    /// This method should perform any necessary setup for the storage backend, such as
    /// establishing database connections or ensuring the existence of required files or schemas.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(StoreError)` if initialisation fails.
    fn initialize(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>>;

    /// Retrieves a value associated with a given key from the store.
    ///
    /// # Arguments
    /// - `key`: A string slice that holds the key for the value to be retrieved.
    ///
    /// # Returns
    /// - `Ok(Some(Value))` if the key exists and the value is successfully retrieved.
    /// - `Ok(None)` if the key does not exist.
    /// - `Err(StoreError)` if there is an error retrieving the value.
    fn get(
        &self,
        key: &str,
    ) -> Pin<
        Box<dyn Future<Output = Result<Option<Value>, StoreError>> + Send + '_>,
    >;

    /// Lists all key-value pairs stored in the store.
    ///
    /// # Returns
    /// - `Ok(Vec<StoreModel>)` containing all the key-value pairs in the store.
    /// - `Err(StoreError)` if there is an error listing the key-value pairs.
    fn list(
        &self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<StoreModel>, StoreError>>
                + Send
                + '_,
        >,
    >;

    /// Sets a value for a given key in the store, with an optional time-to-live (TTL).
    ///
    /// # Arguments
    /// - `key`: The key under which the value is stored.
    /// - `value`: The value to set, represented as a `serde_json::Value`.
    /// - `ttl`: An optional u64 representing the time-to-live in seconds.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(StoreError)` if there is an error setting the value.
    fn set(
        &self,
        key: &str,
        value: Value,
        ttl: Option<u64>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<StoreModel>, StoreError>>
                + Send
                + '_,
        >,
    >;

    /// Removes a value associated with a given key from the store.
    ///
    /// # Arguments
    /// - `key`: A string slice that holds the key for the value to be removed.
    ///
    /// # Returns
    /// - `Ok(())` if the key exists and the value is successfully removed.
    /// - `Err(StoreError)` if there is an error removing the value.
    fn remove(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>>;

    /// Removes multiple values associated with the given keys from the store.
    ///
    /// # Arguments
    /// - `keys`: A slice of string slices representing the keys for the values to be removed.
    ///
    /// # Returns
    /// - `Ok(())` if the values are successfully removed.
    /// - `Err(StoreError)` if there is an error removing the values.
    fn remove_many(
        &self,
        keys: &[&str],
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>>;

    /// Clears all values from the store.
    ///
    /// # Returns
    /// - `Ok(())` if the store is successfully cleared.
    /// - `Err(StoreError)` if there is an error clearing the store.
    fn clear(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + '_>>;
}

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("Failed to connect to the database backend: {0}")]
    ConnectionError(String),

    #[error("Error while serializing or deserializing data")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Database operation failed")]
    DatabaseError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Database query error: {0}")]
    QueryError(String),

    #[error("The requested key was not found")]
    NotFound,

    #[error("An unknown error has occurred")]
    Unknown,
}

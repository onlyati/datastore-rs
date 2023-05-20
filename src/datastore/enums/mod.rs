//! Enum for datastore

use crate::hook::types::{Link, Prefix};

use super::types::{
    ResultWithHook, ResultWithHooks, ResultWithList, ResultWithResult, ResultWithoutResult, Table,
};
use std::sync::mpsc::Sender;

pub mod error;
pub mod pair;

///
/// Specifiy the level for listing key function
///
#[derive(PartialEq, Clone)]
pub enum ListType {
    /// List only the current level
    OneLevel,

    /// List everything under it on recursive way
    All,
}

///
/// Actions for built-in server
///
pub enum DatabaseAction {
    /// Set or update a key-value pair
    Set(Sender<ResultWithoutResult>, String, String),

    /// Get a value for a key
    Get(Sender<ResultWithResult>, String),

    /// Delete a pair
    DeleteKey(Sender<ResultWithoutResult>, String),

    /// Delete a whole table
    DeleteTable(Sender<ResultWithoutResult>, String),

    /// List keys from a route
    ListKeys(Sender<ResultWithList>, String, ListType),

    /// Set new hook
    HookSet(Sender<ResultWithoutResult>, Prefix, Link),

    /// Check that hook exist
    HookGet(Sender<ResultWithHook>, Prefix),

    /// Remove existing hook
    HookRemove(Sender<ResultWithoutResult>, Prefix, Link),

    /// List hooks
    HookList(Sender<ResultWithHooks>, Prefix),
}

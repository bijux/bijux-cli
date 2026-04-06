use crate::RuntimeError;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type AdapterFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, RuntimeError>> + Send + 'a>>;

pub trait AsyncAdapter: Send + Sync {
    type Output: Send + Sync;

    fn execute_async<'a>(&'a self, params: &'a Value) -> AdapterFuture<'a, Self::Output>;
}

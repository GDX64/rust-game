use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

pub struct WrappedActor<A> {
    inner: Arc<Mutex<A>>,
}

impl<A> Clone for WrappedActor<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<A> WrappedActor<A> {
    pub fn new(actor: A) -> Self {
        let wrapped = Self {
            inner: Arc::new(Mutex::new(actor)),
        };
        return wrapped;
    }

    pub async fn with_state<T: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut A) -> T + Send + 'static,
    ) -> T {
        let mut guard = self.inner.lock().await;
        return f(&mut *guard);
    }

    pub fn listener<T: Send + 'static, F: Future<Output = T> + Send + 'static>(
        &self,
        f: impl FnOnce(Self) -> F,
    ) -> tokio::task::JoinHandle<T> {
        let task = tokio::spawn(f(self.clone()));
        return task;
    }
}

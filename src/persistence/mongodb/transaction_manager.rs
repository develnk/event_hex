use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use mongodb::{
    options::{ReadPreference, SelectionCriteria}, Client,
    ClientSession,
};

use crate::errors::EventHexError;
use crate::persistence::transaction::{
    ErasedResult, EventTransactionContext, EventTransactionHandler, EventTransactionManager,
};

/// MongoDB transaction context.
///
/// Wraps a MongoDB [`ClientSession`] so that storage backends can participate
/// in the same transaction.
pub struct MongoContext {
    /// The underlying MongoDB session.
    pub session: ClientSession,
}

impl EventTransactionContext for MongoContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Context for working without transactions (standalone mode).
struct NoopTransactionContext;

impl EventTransactionContext for NoopTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// MongoDB transaction manager.
///
/// Detects whether the connected MongoDB instance supports transactions and
/// transparently falls back to a no-op context in standalone mode.
pub struct MongoTransactionManager {
    client: Arc<Client>,
    use_transactions: bool,
}

impl MongoTransactionManager {
    /// Creates a new manager for the given client.
    ///
    /// Transaction support is detected automatically.
    pub async fn new(client: Arc<Client>) -> Self {
        let use_transactions = Self::detect_transaction_support(&client).await;
        Self { client, use_transactions }
    }

    async fn detect_transaction_support(client: &Client) -> bool {
        match client.start_session().await {
            Ok(mut session) => {
                match session.start_transaction().await {
                    Ok(_) => {
                        let _ = session.abort_transaction().await;
                        true
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}

#[async_trait]
impl EventTransactionManager for MongoTransactionManager {
    async fn run_transaction(&self, handler: EventTransactionHandler) -> Result<ErasedResult, EventHexError> {
        if self.use_transactions {
            // Transaction logic.
            let mut session = self.client.start_session().await.map_err(|e| EventHexError::MongoError(e))?;

            session
                .start_transaction()
                .selection_criteria(SelectionCriteria::ReadPreference(ReadPreference::Primary))
                .await
                .map_err(|e| EventHexError::MongoError(e))?;

            let mut ctx = MongoContext { session };

            let result = handler(&mut ctx).await;

            match result {
                Ok(value) => {
                    ctx.session.commit_transaction().await.map_err(|e| EventHexError::MongoError(e))?;
                    Ok(value)
                }
                Err(e) => {
                    ctx.session.abort_transaction().await.map_err(|e| EventHexError::MongoError(e))?;
                    Err(e)
                }
            }
        } else {
            // Without transaction (standalone mode).
            let mut ctx = NoopTransactionContext;
            handler(&mut ctx).await
        }
    }
}

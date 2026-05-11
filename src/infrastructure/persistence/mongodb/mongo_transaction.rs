use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use mongodb::{
    options::{ReadPreference, SelectionCriteria}, Client,
    ClientSession,
};

use crate::application::ports::transaction::{
    ErasedResult, TransactionContext, TransactionHandler, TransactionManager,
};
use crate::shared_kernel::errors::EventHexError;

// Реализация контекста для Mongo
pub struct MongoContext {
    pub session: ClientSession,
}

// Реализация абстрактного контекста транзакции
impl TransactionContext for MongoContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Контекст для работы без транзакций (standalone mode)
struct NoopTransactionContext;

impl TransactionContext for NoopTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct MongoTransactionManager {
    client: Arc<Client>,
    use_transactions: bool,
}

impl MongoTransactionManager {
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
impl TransactionManager for MongoTransactionManager {
    async fn run_transaction(&self, handler: TransactionHandler) -> Result<ErasedResult, EventHexError> {
        if self.use_transactions {
            // Логика с транзакцией
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
            // Без транзакции (standalone mode)
            let mut ctx = NoopTransactionContext;
            handler(&mut ctx).await
        }
    }
}

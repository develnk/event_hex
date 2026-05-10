use async_trait::async_trait;
use bson::serde_helpers::uuid_1;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use sha2::{Digest, Sha256};
use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;
use crate::errors::DomainEventHandlerError;
use crate::domain::{AggregateRoot, EntityId};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>, // ObjectId MongoDB, опциональный
    #[serde_as(as = "uuid_1::AsBinary")]
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub sequence_number: u32, // Порядковый номер события в рамках агрегата
    pub event_type: String,   // Тип события (строковый идентификатор)
    pub event_data: Value,    // Данные события
    pub metadata: Value,      // Метаданные события
    pub timestamp: DateTime<Utc>, // Время создания события
    pub previous_hash: Vec<u8>, // Хеш предыдущего события в цепочке
}

// Структура хранящаяся в БД
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub event: Event,
    pub hash: Vec<u8>,
}

// Принимает событие, сериализует его и вычисляет SHA-256 хеш.
pub fn calculate_hash(event: &Event) -> Result<Vec<u8>, bincode::error::EncodeError> {
    // Сериализуем событие в байты. Важно использовать детерминированный формат.
    // bincode хорошо подходит для этого.
    let config = bincode::config::standard();
    let serialized_event = bincode::serde::encode_to_vec(event.to_owned(), config)?;

    // Вычисляем SHA-256 хеш
    let mut hasher = Sha256::new();
    hasher.update(serialized_event);
    let hash = hasher.finalize();

    Ok(hash.as_slice().to_vec())
}

impl fmt::Display for StoredEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hash_hex = self.hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();

        let prev_hash_hex = self
            .event
            .previous_hash
            .clone()
            .into_iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        write!(
            f,
            "Event ID: {}\nTimestamp: {}\nPayload: \"{}\"\nPrevious Hash: {}\nCurrent Hash: {}\n---",
            self.event.sequence_number,
            self.event.timestamp,
            self.event.event_data,
            prev_hash_hex,
            hash_hex
        )
    }
}

/// Структура, представляющая новое событие перед сохранением в БД.
/// Используется для передачи данных в метод append.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPreRecord {
    pub metadata: Value,
    pub event: Value,
    pub event_type: String,
}

pub fn get_event_records<T: DomainEvent + Serialize>(events: Vec<&T>) -> Vec<EventPreRecord> {
    events
        .into_iter()
        .map(|event| {
            // @TODO временно пустая метадата.
            let metadata: Value = serde_json::to_value("").unwrap();

            EventPreRecord {
                metadata,
                event: serde_json::to_value(event).unwrap_or(Value::Null),
                event_type: event.event_type_name().to_string(),
            }
        })
        .collect()
}

pub fn convert_event_to_event_pre_record<E: DomainEvent + Serialize>(event: &E) -> EventPreRecord {
    EventPreRecord {
        // @TODO временно пустая метадата.
        metadata: serde_json::Value::Null,
        event: serde_json::to_value(event).unwrap_or(Value::Null),
        event_type: event.event_type_name().to_string(),
    }
}

/// Общий трейт для всех доменных событий.
#[async_trait]
pub trait DomainEvent: Debug + ErasedSerialize + Send + Sync + 'static {
    /// Уникальный ID события
    fn new_event_id(&self) -> Uuid {
        Uuid::now_v7()
    }
    fn aggregate_id(&self) -> EntityId {
        EntityId::new()
    }
    /// Время создания события
    fn occurred_on(&self) -> DateTime<Utc> {
        Utc::now()
    }
    /// Тип события (строковое представление, полезно для логирования/диспатча)
    fn event_type_name(&self) -> String;

    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    // Дополнительный метод для получения ссылки на Any
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
erased_serde::serialize_trait_object!(DomainEvent);

/// Общий трейт для обработчиков событий.
#[async_trait]
pub trait DomainEventHandler<E>: Debug + Send + Sync + 'static
where
    E: DomainEvent,
{
    async fn handle(&self, event: &E);
}

#[async_trait]
pub trait DomainEventHandlerFactory<E>: Send + Sync
where
    E: DomainEvent,
{
    async fn create(&self) -> Result<Box<dyn DomainEventHandler<E>>, DomainEventHandlerError>;
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot<A>
where
    A: AggregateRoot,
{
    #[serde_as(as = "uuid_1::AsBinary")]
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub data: A,
}

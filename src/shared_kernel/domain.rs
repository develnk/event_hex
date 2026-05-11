use crate::shared_kernel::auditable::Auditable;
use crate::shared_kernel::domain_event::{DomainEvent, StoredEvent};
use crate::shared_kernel::errors::DomainError;
#[cfg(feature = "mongo")]
use bson::Bson;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self(Uuid::nil())
    }
}

#[cfg(feature = "mongo")]
impl From<EntityId> for Bson {
    fn from(id: EntityId) -> Self {
        Bson::String(id.as_uuid().to_string())
    }
}

#[cfg(feature = "mongo")]
impl From<EntityId> for bson::Uuid {
    fn from(id: EntityId) -> Self {
        bson::Uuid::from_bytes(id.as_uuid().into_bytes())
    }
}


impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

pub trait AggregateRoot: Sized + Send + Sync + Serialize {
    type Event: DomainEvent + Clone + Serialize + DeserializeOwned;

    /// Возвращает уникальный идентификатор агрегата.
    fn id(&self) -> &EntityId;

    // Для каждого агрегата подготовить своё строковое представление
    fn aggregate_type() -> &'static str;

    /// Применяет событие для восстановления состояния.
    fn apply_event(&mut self, event: Self::Event);

    // Конвертация события хранящегося в БД в доменное событие.
    fn convert_to_domain_event(stored_event: StoredEvent) -> Result<Self::Event, DomainError> {
        let mut val = stored_event.event.payload;
        // Добавляем поле типа прямо в объект данных, чтобы десериализовать весь enum разом
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(stored_event.event.event_type),
            );
        }
        Ok(serde_json::from_value::<Self::Event>(val)?)
    }

    /// Создаём новый агрегат с дефолтным состоянием.
    fn first_state(id: &EntityId) -> Self;

    /// Возвращает текущую версию агрегата (количество применённых событий).
    fn get_version(&self) -> u32;

    fn set_version(&mut self, version: u32);

    fn increment_version(&mut self);
}

// Эта обёртка разделит данные доменной модели и инфраструктурную логику (события).
pub struct AggregateContainer<A: AggregateRoot> {
    pub aggregate: A,
    uncommitted_events: Vec<A::Event>,
    audit: Auditable,
}

impl<A: AggregateRoot> AggregateContainer<A> {
    /// Создание нового контейнера (например, при создании нового агрегата)
    pub fn new(aggregate: A) -> Self {
        Self {
            aggregate,
            uncommitted_events: Vec::new(),
            // TODO Реализовать логику позже.
            audit: Default::default(),
        }
    }

    pub fn version(&self) -> u32 {
        self.aggregate.get_version()
    }

    /// Восстанавливаем агрегат из событий с самого начала.
    /// TODO Переделать stored_events: &Vec<StoredEvent> на events: Vec<A::Event> чтобы не зависеть от структуры хранения события в БД
    pub fn restore_from_history(
        entity_id: &EntityId, stored_events: &Vec<StoredEvent>,
    ) -> Result<Self, DomainError> {
        if stored_events.is_empty() {
            return Err(DomainError::AggregateNotFound {
                aggregate_id: entity_id.as_uuid().into(),
                aggregate_type: String::from("Unknown"),
            });
        }

        let mut aggregate = A::first_state(entity_id);

        for stored_event in stored_events {
            let domain_event = A::convert_to_domain_event(stored_event.to_owned())?;
            aggregate.apply_event(domain_event);
        }

        Ok(Self {
            aggregate,
            uncommitted_events: Vec::new(),
            audit: Default::default(),
        })
    }

    pub fn restore_from_snapshot(
        &mut self, after_events: &Vec<StoredEvent>,
    ) -> Result<(), DomainError> {
        for stored_event in after_events {
            let domain_event = A::convert_to_domain_event(stored_event.to_owned())?;
            self.aggregate.apply_event(domain_event);
        }

        Ok(())
    }

    /// Основной метод для выполнения команд.
    /// Принимает событие, применяет его к агрегату и сохраняет в очередь.
    pub fn push_event(&mut self, event: A::Event) {
        // 1. Применяем к внутреннему состоянию
        self.aggregate.apply_event(event.to_owned());

        // 2. Сохраняем для БД/EventStore
        self.uncommitted_events.push(event);
    }

    pub fn get_events(&self) -> Vec<A::Event> {
        self.uncommitted_events.clone()
    }

    pub fn get_erased_events(&self) -> Vec<Box<dyn DomainEvent>> {
        let events = self.uncommitted_events.clone();
        events
            .into_iter() // потребляем вектор, владение переходит в итератор
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }

    pub fn take_events(&mut self) -> Vec<A::Event> {
        std::mem::take(&mut self.uncommitted_events)
    }

    pub fn take_erased_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        let events = std::mem::take(&mut self.uncommitted_events);
        events
            .into_iter() // потребляем вектор, владение переходит в итератор
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }
}

/*
   Domain layer.
   Core of business logic. Contains aggregates (Order Aggregate, Product Aggregate), entities, value objects, and domain events.
   Does not depend on external layers. Order Read Model and Product Read Model are read-optimized
   data representations used by Query Handlers.
*/
pub mod aggregate;
pub mod event;
pub mod ports;
pub mod service;

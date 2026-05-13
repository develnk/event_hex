# event_hex

<div align="center">
    <a href="https://github.com/develnk/event_hex"><img
        alt="github"
        src="https://img.shields.io/badge/github-develnk/event_hex-228b22?style=for-the-badge&labelColor=555555&logo=github"
        height="25"
    /></a>
    <a href="https://crates.io/crates/event_hex"><img
        alt="crates.io"
        src="https://img.shields.io/crates/v/event_hex.svg?style=for-the-badge&color=e37602&logo=rust"
        height="25"
    /></a>
    <a href="https://docs.rs/event_hex/latest/event_hex/"><img
        alt="docs.rs"
        src="https://img.shields.io/badge/docs.rs-event_hex-3b74d1?style=for-the-badge&labelColor=555555&logo=docs.rs"
        height="25"
    /></a>
    <a href="https://docs.rs/event_hex/latest/event_hex/"><img
        alt="license"
        src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg?style=for-the-badge"
        height="25"
    /></a>
</div>

> Russian version: [README.ru.md](README.ru.md)

## Why event_hex

- **Hexagonal-friendly.** All abstractions are designed with hexagonal architecture layering in mind. All examples are
  built on hexagonal architecture with detailed explanations of what each architectural layer is responsible for.
- **Event Sourcing.** Aggregates generate domain events, EventStore persists them, DomainEventHandler allows you to
  asynchronously apply any necessary logic in response to a published event.
- **Event Store.**  The event store is a central place for storing events - it is the **source of truth**.
  Events cannot be deleted or modified, and the library takes this into account.
    - The event store supports storing the hash of the previous event, creating a chain of linked events. When restoring
      an aggregate's state, these hashes are verified. It is therefore impossible to modify an event without the system
      detecting it.
    - Aggregate snapshots are supported to avoid replaying all events from the beginning.
    - Built-in Concurrency Conflict support. It is impossible to save two events with the same version.
    - `MongoDB` and `PostgreSQL` are supported, and you can also write your own storage implementation.
- **CQRS by default.** CQRS fits perfectly into a hexagonal architecture. An in-memory CommandBus is implemented for
  processing commands, and a QueryBus is implemented for processing queries, both with the ability to register handlers.
- **Async.**  Built on the Tokio async runtime.

## License

Licensed under one of the following licenses:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

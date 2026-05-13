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

> Английская верси: [README.md](README.md)

Набор инструментов на Rust для проектов спроектированных на гексагональной архитектуре
с первоклассной поддержкой **DDD(Domain-Driven Design)**, **Event Sourcing** и **CQRS**

## Почему event_hex

- **Hexagonal-friendly.** Все абстракции реализованы с учётом разделения на слои гексагональной архитектуры.
  Все примеры построены на гексагональной архитектуре с подробным объяснением за что конкретно должен отвечать
  определённый слой архитектуры.
- **Event Sourcing.** Агрегаты генерируют доменные события, EventStore сохраняет их, DomainEventHandler
  позволяет применить асинхронно любую необходимую логику в качестве реакции на выпущенное событие.
- **Event Store.** Хранилище событий это центральное место для хранения событий - **это источник истины**.
  События нельзя удалять либо модифицировать, и библиотека это учитывает.
    - Хранилище событий поддерживает возможность хранить **хеш предыдущего события**, таким образом
      получается цепочка связанных событий. При восстановлении состояния агрегата проверяются эти хеши. Таким образом
      невозможно изменить событие не заметно для системы.
    - Поддерживаются снапшоты агрегата, чтобы не прокручивать все события с самого начала.
    - Встроенная поддержка Concurrency Conflict. Невозможно сохранить два события с одинаковой версией.
    - Поддерживается `MongoDb` и `PostgreSQL`, а также имеется возможность написать свою версию хранилища.
- **CQRS по умолчанию.** CQRS отлично вписывается в гексагональную архитектуру. Для обработки команд реализована
  in-memory шина `CommandBus`, для обработки запросов релизована шина `QueryBus`, с возможностью зарегистрировать
  обработчики.
- **Async.** Основана на async-рантайме **Tokio**

## Лицензия

Лицензировано на условиях одной из лицензий на ваш выбор:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
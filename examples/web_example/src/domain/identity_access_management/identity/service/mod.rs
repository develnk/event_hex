/*
 *
   Domain services.
   Used for:
     Coordination between multiple Aggregates when logic cannot be placed in any single one.
     Implementing complex business logic that is not tied to the state of a specific entity or aggregate.

Example: The TransferService in a banking domain, which coordinates debiting from Account A and crediting to Account B.
Important: Domain Services must not depend on infrastructure (DB, HTTP) or even application (commands/queries).
  They may only depend on other domain elements (Entities, Value Objects, Domain Events, Domain Services).
*/
pub mod access_account_domain_service;

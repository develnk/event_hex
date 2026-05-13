/*
 * Application layer. Implements domain inbound ports.
 * Orchestrates domain logic, gets data from the domain, and passes results to external systems via outbound ports.
 * <p>
 * Application services perform the following functions:
 *   Participation in regulation and distribution of commands, queries, and sagas;
 *   Provision of centralized components (e.g., logging, security, metrics) for
 *     the lower-level domain model;
 *   Execution of calls directed to other bounded contexts.
 *
 * May contain no more than one call to a method that modifies a resource (Outbound Port)
 * Must not contain business logic;
 * May contain branching only to choose the response representation;
    Application services are responsible for delegating commands and queries by calling the regulation
    and coordination model.
    @author Nikolay
*/

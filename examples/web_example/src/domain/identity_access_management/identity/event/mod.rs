/*
 * This package contains Domain Events which include any types of state changes for both the aggregate
 * and the entity within the bounded context.
 * An Event in a bounded context is any operation that publishes aggregate state changes
 * of the bounded context as an event.
 * For example, Commands change the aggregate state, so the action of any command in the bounded context
 * leads to the generation of a corresponding event. Subscribers to these events can be other bounded contexts,
 * belonging to some other external domains.
 *
 * @author Nikolay
 */
pub mod access_account_events;


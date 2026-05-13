/*
* Outgoing Port. Defines what the domain needs from the outside world.
* Implemented in Outgoing Adapter in the infrastructure layer.
* These ports are used by the Application layer through services.
*/
pub mod external_service_ports;
pub mod message_bus_ports;
pub mod read_repository_ports;
pub mod write_repository_ports;

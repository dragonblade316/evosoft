# Evosoft
Evosoft is a lightweight framework/monorepo for evolution robotics robots. 

Evosoft uses a standard zenoh based pubsub system for communicating between nodes. All communication is done through protobufs.

Packages:
Evobridge: Lightweight wrapper around zenoh.
Evotypes: A set of protobuf types that are used for standard communication between nodes.
Evotypes-rs: The rust implementation of Evotypes based on prost..

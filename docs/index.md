# Chaos Symphony

## Architecture

```
           ---------------
client --> |             | <-- simulation
client --> | replication | <-- simulation
client --> |             | <-- simulation
           ---------------
                  |
                  v
             persistence
```

## Client

- ai
- player

## Communication

blocking:

- request: id
  - purchase: e,t
    - response: id
      - purchased: e,t
      - rejected: e,t
- request: id
  - authenticate: i,c
    - response: id
      - authenticated: id
      - unauthenticated: id

non-blocking:

- event: id
  - moved: e,x,y,z

## Handshake

client:

- create connection to replication
  - connection created

replication:

- listen for incoming connections
  - connection created

simulation:

- create connection to replication
  - connection created

## Networking

client:

- inbound:
  - event: payload
- outbound:
  - event: payload
  - request: payload

replication:

- inbound:
  - event: payload
  - request: payload
- outbound:
  - event: payload
  - request: payload

simulation:

- inbound:
  - event: payload
  - request: payload
- outbound:
  - event: payload

## Protocol

request:

- id
- endpoint
- properties

event:

- id
- endpoint
- properties

## Principal

nouns:

- ai
- authority
- player
- replication

scopes:

- world

## Workflows

[subject] [predicate] [object] [context]

client authenticates connection:

- replication validates authentication
  - replication annotates connection

client joins replication:

- replication streams to client

client mutates entity:

- client streams mutation to replication
  - replication streams mutation to authoritative simulation

client mutates entity (optimization):

- (optimization) client applies mutation in memory
- client streams mutation to replication
  - replication streams mutation to authoritative simulation

simulation leaves replication:

- replication assigns zone to simulation
  - replication assigns authority to simulation
    - replication streams to simulation

simulation joins replication:

- replication assigns zone to simulation
  - replication assigns authority to simulation
    - replication streams to simulation

simulation mutates entity:

- simulation stream mutation to replication
  - replication applies mutation in persistence
    - replication applies mutation in memory
    - replication streams mutation to client
      - client applies mutation in memory
    - replication streams mutation to simulation
      - simulation applies mutation in memory

simulation mutates entity (optimization):

- simulation applies mutation in memory (optimization)
- simulation stream mutation to replication
  - replication applies mutation in memory (optimization)
  - replication applies mutation in persistence
  - replication streams mutation to client
    - client applies mutation in memory
  - replication streams mutation to simulation
    - simulation applies mutation in memory

## References

Graph storage:

- http://www.vldb.org/pvldb/vol1/1453965.pdf

  - (Subject, Predicate, Object)
  - (Subject, Object, Predicate)
  - (Object, Predicate, Subject)
  - (Object, Subject, Predicate)
  - (Predicate, Subject, Object)
  - (Predicate, Object, Subject)

- https://www.youtube.com/watch?v=tNgFMBzYcl8

# Replication

## Workflows

[subject] [predicate] [object] [context]

client requests identities: `/request/identities`

- replication streams identities to client: `/response/identities`, `/event/identities`

client request properties for identity: `/request/properties`

- replication streams properties to client: `/response/properties`, `/event/*`

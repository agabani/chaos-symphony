# Replication

## Workflows

[subject] [predicate] [object] [context]

client requests identities: `/request/identities`

- replication streams identities to client: `/response/identities`, `/event/identities`

client request replication for identity: `/request/replication`

- replication streams replication to client: `/response/replication`, `/event/*`

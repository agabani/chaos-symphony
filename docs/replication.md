# Replication

- pull
- push
  - event
    - ai/client
    - simulation
    - replication

## Simulation Sourced Movement

### Client

- recv: event: x moved

### Replication

- recv: event: x moved
  - save: event: x moved
  - send: event: x moved

### Simulation

- send: event: x moved

## Client Sourced Movement

### Client

- send: event: x moved

### Replication

- recv: event: x moved
  - send: event: x moved

### Simulation

- recv: event: x moved

## Implementation - Simulation Sourced Movement

### Server

```rs
fn send_events(query: Query<&T, Changed<T>>) {
    // send to replication
}

fn recv_events(query: Query<&Message<T>>) {}
```

### Replication

```rs
fn send_events(query: Query<&T, Changed<T>>) {}

fn recv_events(query: Query<&Message<T>>) {
    // apply locally
    // send to clients
}
```

### Client

```rs
fn send_events(query: Query<&T, Changed<T>>) {}

fn recv_events(query: Query<&Message<T>>) {
    // apply locally
}
```

## Implementation - Client Sourced Movement

### Client

```rs
fn send_events(query: Query<&T, Changed<T>>) {
    // send to replication
}

fn recv_events(query: Query<&Message<T>>) {}
```

### Replication

```rs
fn send_events(query: Query<&T, Changed<T>>) {}

fn recv_events(query: Query<&Message<T>>) {
    // send to simulation
}
```

### Server

```rs
fn send_events(query: Query<&T, Changed<T>>) {}

fn recv_events(query: Query<&Message<T>>) {
    // apply locally
    // send to replication
}
```

## Implementation (all in one)

### Client

```rs
fn send_events(query: Query<&T, Changed<T>>) {
    // replication.
}

fn recv_events(query: Query<&Message<T>>) {
    // trusted: apply.
}
```

### Replication

```rs
fn send_events(query: Query<&T, Changed<T>>) {
    // client, server, replication.
}

fn recv_events(query: Query<&Message<T>>) {
    // trusted: apply.
    // untrusted: server.
}
```

### Server

```rs
fn send_events(query: Query<&T, Changed<T>>) {
    // replication.
}

fn recv_events(query: Query<&Message<T>>) {
    //--- trusted: apply.
    // untrusted: validate->apply
}
```

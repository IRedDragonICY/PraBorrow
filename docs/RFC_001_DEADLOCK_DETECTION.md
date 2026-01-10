# RFC 001: Distributed Deadlock Detection

## Summary
Implement a global Wait-For Graph (WFG) to detect circular dependencies across the distributed system. Nodes will propagate local WFG edges via Raft headers.

## Motivation
In the PraBorrow architecture, resources are acquired uniquely (Sovereign ownership). A cycle of `try_hire` requests (A waits for B, B waits for C, C waits for A) leads to system-wide deadlock.

## Proposed Design

### 1. Wait-For Graph (WFG)
Each node maintains a local WFG:
- **Nodes**: Resource IDs (`u128`).
- **Edges**: `A -> B` means "Holder of A is waiting for B".

### 2. Propagation
The `CheckProtocol` will be extended:
```rust
pub trait CheckProtocol {
    fn report_dependency(&self, resource_id: u128, waiting_for: u128);
}
```

### 3. Detection Algorithm
An asynchronous `DeadlockDetector` service runs on the Leader:
1. Aggregates WFGs from all Follower heartbeats.
2. Runs a cycle detection algorithm (DFS).
3. If cycle found -> Preempts the youngest transaction (returns `LeaseError::Deadlock`).

## Integration Plan
- Add `wfg` field to `RaftNode`.
- Piggyback WFG updates on `AppendEntries`.

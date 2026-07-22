# Token Vesting Event Schema

Schema version: **1.0.0**

This document defines the events emitted by the token vesting contract for
indexers, relayers, and other off-chain consumers. It describes the native
Soroban topic and data values written by `Env::events().publish`. The examples
show decoded values, not the base64-encoded XDR returned by Stellar RPC.

## Common envelope

Stellar RPC supplies the contract ID and ledger metadata outside the event
payload. For events from this contract, decode the contract event as follows:

- `topics[0]` is the event name, encoded as a Soroban `Symbol`.
- `topics[1]` is the actor address used to filter the event.
- `data` is either one Soroban value or a positional tuple, as specified below.
- `Address` values can represent Stellar accounts or contracts.
- `i128` values should be retained as signed 128-bit integers. JSON consumers
  should serialize them as decimal strings to avoid precision loss.
- All `u64` time values are Unix timestamps in seconds.

Field names in the decoded examples are descriptive. The on-chain data tuples
are positional and do not contain those names.

## `initialize`

Emitted once when the contract is initialized.

| Location | Position | Field | Soroban type | Meaning |
| --- | ---: | --- | --- | --- |
| Topic | 0 | `event` | `Symbol` | Always `initialize` |
| Topic | 1 | `admin` | `Address` | Administrator that initialized the contract |
| Data | value | `token` | `Address` | Stellar Asset Contract used by all schedules |

Decoded example:

```json
{
  "event": "initialize",
  "admin": "GADMIN...",
  "token": "CTOKEN..."
}
```

Source layout:

```rust
topics = (Symbol("initialize"), admin)
data = token
```

## `create_vesting`

Emitted after a new schedule is stored and its tokens have been transferred to
the contract.

| Location | Position | Field | Soroban type | Meaning |
| --- | ---: | --- | --- | --- |
| Topic | 0 | `event` | `Symbol` | Always `create_vesting` |
| Topic | 1 | `creator` | `Address` | Address that funded the schedule |
| Data | 0 | `id` | `u64` | Contract-local vesting schedule ID |
| Data | 1 | `beneficiary` | `Address` | Address allowed to claim vested tokens |
| Data | 2 | `amount` | `i128` | Total token amount deposited |
| Data | 3 | `cliff_time` | `u64` | Timestamp when linear vesting begins |
| Data | 4 | `end_time` | `u64` | Timestamp when the full amount is vested |

Decoded example:

```json
{
  "event": "create_vesting",
  "creator": "GCREATOR...",
  "id": 7,
  "beneficiary": "GBENEFICIARY...",
  "amount": "1000000000",
  "cliff_time": 1788192000,
  "end_time": 1819728000
}
```

Source layout:

```rust
topics = (Symbol("create_vesting"), creator)
data = (id, beneficiary, amount, cliff_time, end_time)
```

## `claim`

Emitted after claimable tokens are transferred to the beneficiary and the
schedule's claimed amount is updated.

| Location | Position | Field | Soroban type | Meaning |
| --- | ---: | --- | --- | --- |
| Topic | 0 | `event` | `Symbol` | Always `claim` |
| Topic | 1 | `beneficiary` | `Address` | Address that received the tokens |
| Data | 0 | `vesting_id` | `u64` | Claimed schedule ID |
| Data | 1 | `amount` | `i128` | Amount transferred by this claim |

Decoded example:

```json
{
  "event": "claim",
  "beneficiary": "GBENEFICIARY...",
  "vesting_id": 7,
  "amount": "250000000"
}
```

Source layout:

```rust
topics = (Symbol("claim"), beneficiary)
data = (vesting_id, amount)
```

The contract names the second tuple value `claimable` internally. It represents
the amount transferred by this claim, so the schema exposes it as `amount`.

## `revoke`

Emitted after a schedule is marked revoked and any unclaimed return amount is
transferred to its creator.

| Location | Position | Field | Soroban type | Meaning |
| --- | ---: | --- | --- | --- |
| Topic | 0 | `event` | `Symbol` | Always `revoke` |
| Topic | 1 | `creator` | `Address` | Schedule creator that revoked the schedule |
| Data | 0 | `vesting_id` | `u64` | Revoked schedule ID |
| Data | 1 | `unclaimed_amount` | `i128` | Amount returned to the creator |

Decoded example:

```json
{
  "event": "revoke",
  "creator": "GCREATOR...",
  "vesting_id": 7,
  "unclaimed_amount": "600000000"
}
```

Source layout:

```rust
topics = (Symbol("revoke"), creator)
data = (vesting_id, unclaimed_amount)
```

The contract names the second tuple value `unclaimed` internally.

## Indexing guidance

1. Filter contract events by the deployed vesting contract ID.
2. Decode `topics[0]` and dispatch on the four event symbols above.
3. Treat `topics[1]` as the indexed actor and decode the data value according to
   the matching positional schema.
4. Key schedules by both contract ID and vesting ID. IDs are unique only within
   one contract instance.
5. Order events by ledger sequence and transaction/event position. Do not rely
   on ingestion time.
6. Keep raw XDR alongside decoded fields so records can be replayed after an
   indexer upgrade.

## Versioning policy

Schema 1.0.0 documents the event layouts implemented by the current contract.
The schema follows semantic versioning:

- Patch: clarifications that do not change decoding.
- Minor: a new event or an optional, backward-compatible consumer feature.
- Major: a changed event symbol, topic order, data tuple order, or field type.

Consumers should reject an unexpected tuple length instead of guessing at a
new layout. Contract deployments should record the schema version used by their
code release, because the version is documented off-chain and is not embedded
in each event.

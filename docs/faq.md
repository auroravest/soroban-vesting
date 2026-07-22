# Vesting FAQ

## What happens if I lose my key?

The contract cannot recover or replace a lost key. Claiming requires authorization
from the beneficiary address recorded in the vesting schedule. If the beneficiary
loses that address's signing key, the beneficiary cannot claim through another
address.

The same limitation applies to the creator: only the creator address recorded in
the schedule can revoke it. Keep signing keys backed up securely and test the
recovery process before funding a schedule.

## Can I speed up vesting?

No. A schedule's beneficiary, total amount, cliff time, and end time are fixed when
the schedule is created. The contract has no function for shortening the cliff,
changing the end time, or releasing tokens early.

The beneficiary may claim any amount that has already vested, but cannot claim
future tokens. The creator may revoke a schedule, but revocation is not an
acceleration mechanism and permanently prevents further claims from that schedule.

## How is the vested amount calculated?

For a schedule with total amount `A`, cliff time `C`, end time `E`, and current
ledger timestamp `T`:

- Before the cliff (`T < C`), the vested amount is `0`.
- From the cliff through the linear period (`C <= T < E`), the vested amount is
  `A * (T - C) / (E - C)`.
- At or after the end (`T >= E`), the full amount `A` is vested.

The contract uses integer arithmetic, so a fractional result is rounded down.
The currently claimable amount is the vested amount minus tokens already claimed,
with a minimum of zero. Use `get_vesting` to inspect both vested and claimable
amounts, or `get_claimable` for the claimable amount alone.

## What happens on revocation?

Only the schedule's creator can revoke it. Revocation marks the schedule as revoked,
returns the contract's calculated refund to the creator, emits a `revoke` event,
and permanently blocks the beneficiary from making another claim from that
schedule.

The current contract calculates the creator's refund as the total amount minus the
amount vested at revocation, then minus the amount already claimed. Because future
claims are blocked after revocation, participants should review the schedule's
vested, claimed, and claimable amounts before the creator revokes it.

# Quadratic Voting Parachain

A [Cumulus](https://github.com/paritytech/cumulus/)-based Substrate node, which allows for quadratic voting with identities.

## Protocol Design

The quadratic voting protocol takes hints from Cardano's Governance Model, and is designed to be compatible with the [Spec](https://mdpi-res.com/d_attachment/information/information-13-00305/article_deploy/information-13-00305-v3.pdf?version=1655859835) defined.

All stages of voting are restricted to the users that have an identity - and have not been slashed by the registrar.

### Stages of Voting

0. A member of the technical committee kicks off the voting round. In the future, this can be swapped out for actual governance OR have it automated when the previous voting round is finalized.

#### Proposal Phase (~1 week)

1. A proposer creates a proposal and submits it to the chain, with a bond which will be returned to them upon vote execution. Each identity can only create one proposal per voting round.

#### Pre Voting Phase (~1 week)

1. The proposals are grouped into buckets of size `BUCKET_SIZE` (5), by using randomness from BABE.
2. Voters register to be a part of any bucket they are interested in, with the stake proportional to how many votes they would like. This stake will be returned to them upon vote execution.


#### Voting Phase (~1 week)

1. Voters can begin to assign their votes to the proposals in the buckets they registered for.


#### Post Voting Phase (~3 days)

1. The results are tallied by the voters for the buckets they registered for.
2. A challenge period exists, during which any voter can challenge the results.
3. The results are tallied again, and if there is a discrepency, the bucket is cancelled, and all voters are refunded, except for those who equivocated.

#### Enactment Phase (~1 week)

1. The vote is enacted by the technical committee
2. The bonds are returned to the proposers
3. The stake is returned to the voters


## Technical Details

### Pre-Existing Pallets used

1. `cumulus-pallet-parachain-system` - The main pallet for the parachain.
2. `pallet-balances`
3. `pallet-transaction-payment`
4. `pallet-authorship`
5. `pallet-collator-selection`
6. `pallet-session`
7. `pallet-aura`
8. `pallet-aura-ext`
9. `cumulus-pallet-xcmp-queue`
10. `pallet-xcm`
11. `cumulus-pallet-xcm`
12. `cumulus-pallet-dmp-queue`
13. `pallet-identity`
14. `pallet-collective`

### Self Made Pallets

1. `pallet-quadratic-voting`

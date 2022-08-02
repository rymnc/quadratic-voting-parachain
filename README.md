# Quadratic Voting Parachain

A [Cumulus](https://github.com/paritytech/cumulus/)-based Substrate node, which allows for quadratic voting with identities.

## Protocol Design

The quadratic voting protocol takes hints from Cardano's Governance Model, and is designed to be compatible with the [Spec](https://mdpi-res.com/d_attachment/information/information-13-00305/article_deploy/information-13-00305-v3.pdf?version=1655859835) defined.

All stages of voting are restricted to the users that have an identity - and have not been slashed by the registrar.

### Stages of Voting

0. A member of the technical committee kicks off the voting round.

#### Proposal Phase (~1 week)

1. A proposer creates a proposal and submits it to the chain, with a bond which will be returned to them upon vote execution.

#### Pre Voting Phase (~1 week)

1. The proposals are grouped into buckets of size `BUCKET_SIZE` (5), by using randomness from BABE.
2. Voters register to be a part of any bucket they are interested in, with the stake proportional to how many votes they would like. This stake will be returned to them upon vote execution.


#### Voting Phase (~1 week)

1. Voters can begin to assign their votes to the proposals in the buckets they registered for


#### Post Voting Phase (~3 days)

1. The results are tallied

#### Enactment Phase (~1 week)

1. The vote is enacted by the technical committee
2. The bonds are returned to the proposers
3. The stake is returned to the voters

## Technical Details

The cumulus parachain template was used to allow this chain to use BABE's randomness. Eventually, we should be able to send proposal finalizations to other chains
via XCM. This could enable this parachain to be used as a "Hub" for multiple parachains trying to achieve consensus on proposals that they are mutually affected by.

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
15. `pallet-balances`
16. `pallet-randomness-collective-flip`

### Self Made Pallets

1. `pallet-quadratic-voting`


## Future work and current constraints

1. We are using the [Collective Flip Pallet](https://paritytech.github.io/substrate/master/pallet_randomness_collective_flip/index.html), which is not recommended for production since it is unsafe.
Significant headway has been made in using the relay chain's randomness (provided by BABE) [here](https://github.com/paritytech/cumulus/issues/463).
However, it is important to note that BABE's randomness is known by block producers [2 epochs in advance](https://github.com/paritytech/substrate/blob/master/frame/babe/src/randomness.rs#L83-L120)
For a truly censorship-resistant source of randomness, a public protocol like [drand](https://drand.love/) can be used, in tandem with OCW. Upon further investigation, to use drand with substrate, a network module must be created, similar to
[bitswap](https://github.com/paritytech/substrate/blob/84cc128a6edc1c87b68954e6d64407ee36be45c1/client/network/src/bitswap.rs#L1). Bitswap is primarily intended for use in OCW to
store/get block data. A generic libp2p node would allow a much more connected web of networks, with substrate nodes deciding which p2p networks to communicate with. OCW's natively
allow for bridging data from these p2p networks to the chain state via transactions.

	Therefore, to have an additional source of randomness (on a common-good parachain?), the following steps should be completed -
   - Create the drand libp2p module in substrate's client code
   - Expose it to use with OCW
   - Have an OCW fetch the latest randomness, verify it, and post a transaction to the chain
   - Use the randomness from the runtime

	This randomness could then be used to group proposals into buckets, which would prevent the block weight from being completely used due to the relation of proposals to the number of voters.

2. Creation of voting rounds is restricted to a member of the technical commitee. This should be open to anyone with an identity with a "reputation" greater than a set threshold
This would require extending the identity pallet to have reputation.

3. Dispatchable proposals are currently not implemented. When a proposal is finalized, we just deposit an Event. In the future, we should implement the dispatchable similar to how it is done in the [collective pallet](https://github.com/paritytech/substrate/blob/master/frame/collective/src/lib.rs#L184-L187).

4. Votes are submitted simply as an aye or nay. This could lead to censorship since collators are able to look at the voters decision.
A commit-reveal method could be used, however that would require an additional overhead on the client side, which needs to be connected to the internet throughout the voting and post-voting phases, to ensure that the vote is not invalidated.
There is an old pallet for [zk](https://github.com/Polkadex-Substrate/megaclite) which could be used for zk-based voting, but this needs more research in the context of quadratic voting.

5. Currently, the voters themselves tally the votes and submit disputes if the node submits an invalid tally. To follow Cardano's Governance system, ideally we should have a per-bucket "committee",
which is selected by an inequality -

	$$ hash(AccountId, Signature(Randomness)) ≤ Difficulty $$


	The per-bucket committee can be selected after voters register to be a part of the bucket. If the voter satisfies an inequality provided by BABE's randomness (or another source), they can apply to be a committee member for the given bucket with a bond, which is returned to them upon vote execution.
	Committee members are tasked with tallying the votes, which reduces overhead on regular voters if compute-heavy cryptographic primitives (like ZK) are used to wrap votes.

	This committee is then open to disputes and challenges by regular voters.

6. There are no integrity checks for the constants, they should be implemented in the future.

7. There are asserts in tests which don't need to exist, they should be removed in the future.

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		bounded_vec,
		pallet_prelude::*,
		storage::bounded_vec::BoundedVec,
		traits::{Currency, EnsureOrigin, Randomness, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use rand::{seq::SliceRandom, SeedableRng}; // 0.6.5
	use rand_chacha::ChaChaRng;
	use scale_info::TypeInfo; // 0.1.1

	// Ideally, these would be in a primitives directory
	pub type VotingRoundId = u32;
	pub type ProposalCount = u32;
	pub type MaxVotes = u32;
	pub type BucketId = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type BlocksForVotingPhase: Get<BlockNumberFor<Self>>;
		type OneBlock: Get<BlockNumberFor<Self>>;
		type BlocksForPostVotingPhase: Get<BlockNumberFor<Self>>;
		type BlocksForPreVotingPhase: Get<BlockNumberFor<Self>>;
		type BlocksForProposalPhase: Get<BlockNumberFor<Self>>;
		type BlocksForEnactmentPhase: Get<BlockNumberFor<Self>>;
		#[pallet::constant]
		type MaxProposals: Get<ProposalCount>;
		type Token: ReservableCurrency<Self::AccountId>;
		type BondForVotingRound: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
		type BondForProposal: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
		type BondForVoting: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
		type ManagerOrigin: EnsureOrigin<Self::Origin>;
		#[pallet::constant]
		type MaxVotes: Get<MaxVotes>;
		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		type BucketSize: Get<BucketId>;
	}

	type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
	type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
	<<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VotingPhases {
		Proposal,
		PreVoting,
		Voting,
		PostVoting,
		Enactment,
		Finalized,
	}

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VoteDirection {
		Aye,
		Nay
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct VotingPhaseData<BlockNumber> {
		pub start_block: BlockNumber,
		pub end_block: BlockNumber,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct VotingRoundMetadata<AccountId, BlockNumber> {
		pub initializer: AccountId,
		pub proposal_phase: VotingPhaseData<BlockNumber>,
		pub previous_round_id: VotingRoundId,
		pub pre_voting_phase: VotingPhaseData<BlockNumber>,
		pub voting_phase: VotingPhaseData<BlockNumber>,
		pub post_voting_phase: VotingPhaseData<BlockNumber>,
		pub enactment_phase: VotingPhaseData<BlockNumber>,
		pub finalized_block: BlockNumber,
		pub phase: VotingPhases,
	}

	#[derive(
		Clone, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(MaxVotes))]
	#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
	pub struct Proposal<AccountId, Balance, MaxVotes>
	where
		MaxVotes: Get<u32>,
	{
		pub initializer: AccountId,
		pub ayes: BoundedVec<Balance, MaxVotes>,
		pub nays: BoundedVec<Balance, MaxVotes>,
		pub bucket_id: Option<BucketId>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	pub(super) type VotingRounds<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		VotingRoundId,
		VotingRoundMetadata<AccountIdFor<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn latest_voting_round)]
	pub(super) type LatestVotingRound<T: Config> = StorageValue<_, VotingRoundId>;

	#[pallet::storage]
	#[pallet::getter(fn proposals_for_voting_round)]
	pub(super) type ProposalsForVotingRound<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		VotingRoundId,
		BoundedVec<Proposal<T::AccountId, BalanceOf<T>, T::MaxVotes>, T::MaxProposals>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn voters_for_bucket)]
	pub(super) type VotersForBucket<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, VotingRoundId>,
			NMapKey<Blake2_128Concat, BucketId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		// total bond, remaining bond
		(BalanceOf<T>, BalanceOf<T>),
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Emits the voting round id
		ProposalPhaseStarted(VotingRoundId),
		PreVotingPhaseStarted(VotingRoundId),
		VotingPhaseStarted(VotingRoundId),
		PostVotingPhaseStarted(VotingRoundId),
		EnactmentPhaseStarted(VotingRoundId),
		Finalized(VotingRoundId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// voting round not found
		VotingRoundNotFound,
		// Proposal phase cannot be started because the previous voting round is still active
		ProposalPhaseCannotStart,
		// Invalid user tries to start the proposal phase
		NoPermissionToStartProposalPhase,
		// Invalid proposal
		ProposalNotFound,
		// No proposals in voting round
		NoProposals,
		// Storage Overflow
		StorageOverflow,
		// Identity not found
		IdentityNotFound,
		// Invalid bucket id
		InvalidBucketId,
		// only allowed in proposal phase
		CanCallOnlyDuringProposalPhase,
		// only allowed in prevoting phase
		CanCallOnlyDuringPreVotingPhase,
		// only allowed in voting phase
		CanCallOnlyDuringVotingPhase,
		// no tokens bonded to vote
		NoTokensBonded,
		// user tried to vote more than their bond
		CannotVoteMoreThanBond,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub voting_round_id: VotingRoundId,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { voting_round_id: 0 }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			let voting_round_id = &self.voting_round_id;
			<LatestVotingRound<T>>::put(*voting_round_id);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let mut weight: Weight = 0;
			let voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				// this will happen only when the pallet is initialized for the first time
				None => 0,
			};
			weight += 1;
			if voting_round_id == 0 {
				return weight
			}

			let mut voting_round =
				VotingRounds::<T>::get(voting_round_id).expect("Past voting round must exist");

			// state machine for voting rounds
			match voting_round.phase {
				VotingPhases::Proposal => {
					if block_number == voting_round.proposal_phase.end_block {
						// group proposals into buckets of k size + transition state
						// group proposals
						let random = T::Randomness::random(&block_number.encode());

						// shuffle with random. Not sure if its possible to shuffle in place, so fetching all and shuffling by hand
						// usage of sort_by was explored
						let proposals = ProposalsForVotingRound::<T>::get(voting_round_id);

						// we let the state change regardless of proposals being empty
						if proposals.is_some() {
							let mut z: [u8; 32] = [0u8; 32];
							let random_encoded = random.0.encode();
							for i in random_encoded {
								z.fill(i);
							}
							let mut rng = ChaChaRng::from_seed(z); // Vec<u8> => [u8; 32]
							let mut unbounded = Vec::with_capacity(T::MaxProposals::get() as usize);
							for ele in proposals.expect("qed") {
								unbounded.push(ele);
							}
							unbounded.shuffle(&mut rng);
							for i in 0..unbounded.len() {
								let bucket_id = T::BucketSize::get() % ((i as BucketId) + 1);
								let who = unbounded[i].initializer.clone();
								unbounded[i] = Proposal::<T::AccountId, BalanceOf<T>, T::MaxVotes> {
									initializer: who,
									ayes: bounded_vec![],
									nays: bounded_vec![],
									bucket_id: Some(bucket_id as BucketId),
								};
							}
							let randomized = BoundedVec::<
								Proposal<T::AccountId, BalanceOf<T>, T::MaxVotes>,
								T::MaxProposals,
							>::truncate_from(unbounded);
							ProposalsForVotingRound::<T>::set(voting_round_id, Some(randomized));
						}

						// transition state
						voting_round.phase = VotingPhases::PreVoting;
						VotingRounds::<T>::set(voting_round_id, Some(voting_round));
					}
				},
				VotingPhases::PreVoting => {
					if block_number == voting_round.pre_voting_phase.end_block {
						// transition state
						voting_round.phase = VotingPhases::Voting;
						VotingRounds::<T>::set(voting_round_id, Some(voting_round));
					}
				},
				VotingPhases::Voting => {
					if block_number == voting_round.voting_phase.end_block {
						// tally votes + transition state
						todo!();
					}
				},
				VotingPhases::PostVoting => {
					if block_number == voting_round.post_voting_phase.end_block {
						// return bond and stake + transition state
						todo!();
					}
				},
				VotingPhases::Enactment => {
					if block_number == voting_round.enactment_phase.end_block {
						// transition state
						voting_round.phase = VotingPhases::Finalized;
						VotingRounds::<T>::set(voting_round_id, Some(voting_round));
					}
				},
				VotingPhases::Finalized => (),
			};
			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// The following function starts a new proposal round, provided the origin
		// belongs to the technical committee,
		// and the previous voting round has "finalized"
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn start_voting_round(origin: OriginFor<T>) -> DispatchResult {
			// check if the user is a member of the technical committee
			T::ManagerOrigin::ensure_origin(origin.clone())?;
			let who = ensure_signed(origin)?;

			let latest_voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				// this will happen only when the pallet is initialized for the first time
				None => 0,
			};

			if latest_voting_round_id > 0 {
				// Check if the previous voting round has completed
				let past_voting_round = match VotingRounds::<T>::get(latest_voting_round_id) {
					Some(metadata) => metadata,
					None => Err(Error::<T>::VotingRoundNotFound)?,
				};

				// check if phase is finalized
				if past_voting_round.phase != VotingPhases::Finalized {
					Err(Error::<T>::ProposalPhaseCannotStart)?
				}
			}

			// bond some tokens to the voting round
			let bond = T::BondForVotingRound::get();

			T::Token::reserve(&who, bond)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			// start the proposal phase
			let next_voting_round_id =
				latest_voting_round_id.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			let next_voting_round_metadata =
				make_voting_round_metadata::<T>(who, current_block, latest_voting_round_id)?;

			VotingRounds::<T>::insert(next_voting_round_id, next_voting_round_metadata.clone());
			LatestVotingRound::<T>::put(next_voting_round_id);

			Self::deposit_event(Event::ProposalPhaseStarted(next_voting_round_id));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_proposal(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// ensure those who create proposals are backed by identities
			match pallet_identity::pallet::Pallet::<T>::identity(&who) {
				Some(_) => {},
				None => Err(Error::<T>::IdentityNotFound)?,
			};

			let voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			let voting_round = match VotingRounds::<T>::get(voting_round_id) {
				Some(metadata) => metadata,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			match voting_round.phase {
				VotingPhases::Proposal => {
					// check if proposals exist
					let proposals = ProposalsForVotingRound::<T>::get(voting_round_id);
					let new_proposal = Proposal::<T::AccountId, BalanceOf<T>, T::MaxVotes> {
						initializer: who.clone(),
						ayes: bounded_vec![],
						nays: bounded_vec![],
						bucket_id: None,
					};

					if !proposals.is_some() {
						let mut new_proposal_list: BoundedVec<
							Proposal<T::AccountId, BalanceOf<T>, T::MaxVotes>,
							T::MaxProposals,
						> = bounded_vec![];
						// wouldn't actually error out
						new_proposal_list
							.try_insert(0 as usize, new_proposal)
							.map_err(|_| Error::<T>::StorageOverflow)?;
						ProposalsForVotingRound::<T>::set(voting_round_id, Some(new_proposal_list));
					} else {
						ProposalsForVotingRound::<T>::try_append(voting_round_id, new_proposal)
							.map_err(|_| Error::<T>::StorageOverflow)?;
					}
				},
				VotingPhases::PreVoting |
				VotingPhases::Voting |
				VotingPhases::PostVoting |
				VotingPhases::Enactment |
				VotingPhases::Finalized => Err(Error::<T>::CanCallOnlyDuringProposalPhase)?,
			};

			// bond according to proposal cost
			T::Token::reserve(&who, T::BondForProposal::get())?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register_to_vote(origin: OriginFor<T>, bucket_id: BucketId, votes: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if bucket_id > T::BucketSize::get() {
				Err(Error::<T>::InvalidBucketId)?
			}

			// ensure those who register are backed by identities
			match pallet_identity::pallet::Pallet::<T>::identity(&who) {
				Some(_) => {},
				None => Err(Error::<T>::IdentityNotFound)?,
			};

			let voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			let voting_round = match VotingRounds::<T>::get(voting_round_id) {
				Some(metadata) => metadata,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			match voting_round.phase {
				VotingPhases::PreVoting => {
					match ProposalsForVotingRound::<T>::get(voting_round_id) {
						Some(proposals) => proposals,
						None => Err(Error::<T>::NoProposals)?,
					};
					VotersForBucket::<T>::insert((voting_round_id, bucket_id, &who), (votes, votes));
				},
				VotingPhases::Proposal |
				VotingPhases::Voting |
				VotingPhases::PostVoting |
				VotingPhases::Enactment |
				VotingPhases::Finalized => Err(Error::<T>::CanCallOnlyDuringPreVotingPhase)?,
			};

			T::Token::reserve(&who, votes)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote(origin: OriginFor<T>, proposal_id: ProposalCount, vote: BalanceOf<T>, direction: VoteDirection) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			let voting_round = match VotingRounds::<T>::get(voting_round_id) {
				Some(metadata) => metadata,
				None => Err(Error::<T>::VotingRoundNotFound)?,
			};

			match voting_round.phase {
				VotingPhases::Voting => {
					let mut proposals = match ProposalsForVotingRound::<T>::get(voting_round_id) {
						Some(proposals) => proposals,
						None => Err(Error::<T>::NoProposals)?,
					};

					let proposal = match proposals.get_mut(proposal_id as usize) {
						Some(proposal) => proposal,
						None => Err(Error::<T>::ProposalNotFound)?,
					};

					let attached_bucket_id = proposal.bucket_id.expect("qed");

					let mut bonded_tokens = match VotersForBucket::<T>::get((voting_round_id, attached_bucket_id, &who)) {
						Some(tokens) => tokens,
						None => Err(Error::<T>::NoTokensBonded)?,
					};

					// check if vote is greater than the remaining bond
					if vote > bonded_tokens.1 {
						Err(Error::<T>::CannotVoteMoreThanBond)?
					}

					// we accept the vote now
					let _ = match direction {
						VoteDirection::Aye => {
							proposal.ayes.try_push(vote).map_err(|_| Error::<T>::StorageOverflow)?;
							bonded_tokens = (bonded_tokens.0, bonded_tokens.1 - vote);
						},
						VoteDirection::Nay => {
							proposal.nays.try_push(vote).map_err(|_| Error::<T>::StorageOverflow)?;
							bonded_tokens = (bonded_tokens.0, bonded_tokens.1 - vote);
						}
					};

					proposals[proposal_id as usize] = Proposal::<T::AccountId, BalanceOf<T>, T::MaxVotes> {
						initializer: proposal.initializer.clone(),
						ayes: proposal.ayes.clone(),
						nays: proposal.nays.clone(),
						bucket_id: proposal.bucket_id,
					};
					ProposalsForVotingRound::<T>::set(voting_round_id, Some(proposals));
					VotersForBucket::<T>::set((voting_round_id, attached_bucket_id, &who), Some(bonded_tokens));
				},
				VotingPhases::Proposal |
				VotingPhases::PreVoting |
				VotingPhases::PostVoting |
				VotingPhases::Enactment |
				VotingPhases::Finalized => Err(Error::<T>::CanCallOnlyDuringVotingPhase)?,
			};

			Ok(())
		}
	}

	pub fn make_voting_round_metadata<T: Config>(
		initiator: AccountIdFor<T>,
		start_block: BlockNumberFor<T>,
		previous_round_id: VotingRoundId,
	) -> Result<VotingRoundMetadata<AccountIdFor<T>, BlockNumberFor<T>>, Error<T>> {
		let proposal_start = start_block;
		let proposal_end = start_block + T::BlocksForProposalPhase::get();

		let pre_voting_start = proposal_end + T::OneBlock::get();
		let pre_voting_end = pre_voting_start + T::BlocksForPreVotingPhase::get();

		let voting_start = pre_voting_end + T::OneBlock::get();
		let voting_end = voting_start + T::BlocksForVotingPhase::get();

		let post_voting_start = voting_end + T::OneBlock::get();
		let post_voting_end = post_voting_start + T::BlocksForPostVotingPhase::get();

		let enactment_start = post_voting_end + T::OneBlock::get();
		let enactment_end = enactment_start + T::BlocksForEnactmentPhase::get();

		let finalized = enactment_end + T::OneBlock::get();

		return Ok(VotingRoundMetadata::<AccountIdFor<T>, BlockNumberFor<T>> {
			initializer: initiator,
			phase: VotingPhases::Proposal,
			previous_round_id,
			proposal_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: proposal_start,
				end_block: proposal_end,
			},
			pre_voting_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: pre_voting_start,
				end_block: pre_voting_end,
			},
			voting_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: voting_start,
				end_block: voting_end,
			},
			post_voting_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: post_voting_start,
				end_block: post_voting_end,
			},
			enactment_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: enactment_start,
				end_block: enactment_end,
			},
			finalized_block: finalized,
		})
	}
}

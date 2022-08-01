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
		pallet_prelude::*,
		traits::{Currency, EnsureOrigin, ReservableCurrency},
	};
	use frame_support::sp_runtime::traits::CheckedAdd;
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	pub type VotingRoundId = u32;

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
		type Token: ReservableCurrency<Self::AccountId>;
		type BondForVotingRound: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
		type ManagerOrigin: EnsureOrigin<Self::Origin>;
	}

	type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
	type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VotingPhases {
		Proposal,
		PreVoting,
		Voting,
		PostVoting,
		Enactment,
		Finalized,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct VotingPhaseData<BlockNumber> {
		pub start_block:  BlockNumber,
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
		pub phase: VotingPhases,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	pub(super) type VotingRounds<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		VotingRoundId,
		// initiator, start_block, end_block, previous_round_id
		VotingRoundMetadata<AccountIdFor<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn latest_voting_round)]
	pub(super) type LatestVotingRound<T: Config> = StorageValue<_, VotingRoundId>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Emits the voting round id
		ProposalPhaseStarted(VotingRoundId),
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
		// Storage Overflow
		StorageOverflow,
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
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let mut weight: Weight = 0;
			let latest_voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				// this will happen only when the pallet is initialized for the first time
				None => 0,
			};
			weight += 1;
			if latest_voting_round_id == 0 {
				return weight;
			}

			let past_voting_round_opt = VotingRounds::<T>::get(latest_voting_round_id);

			if past_voting_round_opt.is_some() {
				let _ = past_voting_round_opt.unwrap();
			}

			// check if phase is reaching next round
			// TODO
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
			// T::ManagerOrigin::ensure_origin(origin.clone())?;
			let who = ensure_signed(origin)?;

			let latest_voting_round_id = match LatestVotingRound::<T>::get() {
				Some(id) => id,
				// this will happen only when the pallet is initialized for the first time
				None => 0,
			};

			sp_std::if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
    			println!("{}", latest_voting_round_id);
			}

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

			sp_std::if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
    			println!("Bond: {:#?}", bond);
				println!("Token balance: {:#?}", T::Token::free_balance(&who));
			}

			T::Token::reserve(&who, bond)?;

			let current_block = <frame_system::Pallet<T>>::block_number();

			sp_std::if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
    			println!("Current block: {:#?}", current_block);
			}

			// start the proposal phase
			let next_voting_round_id =
				latest_voting_round_id.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			let next_voting_round_metadata =
				make_voting_round_metadata::<T>(who, current_block, latest_voting_round_id)?;

			VotingRounds::<T>::insert(next_voting_round_id, next_voting_round_metadata);
			LatestVotingRound::<T>::put(next_voting_round_id);

			Self::deposit_event(Event::ProposalPhaseStarted(next_voting_round_id));

			// Return a successful DispatchResultWithPostInfo
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
			voting_phase: VotingPhaseData::<BlockNumberFor<T>>  {
				start_block: voting_start,
				end_block: voting_end,
			},
			post_voting_phase: VotingPhaseData::<BlockNumberFor<T>> {
				start_block: post_voting_start,
				end_block: post_voting_end,
			},
		});
	}
}

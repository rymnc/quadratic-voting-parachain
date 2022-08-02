use frame_support::pallet_prelude::EnsureOrigin;
use frame_support::parameter_types;
use crate as quadratic_voting_pallet;
use frame_support::traits::{ConstU128, ConstU16, ConstU32, ConstU64, OnInitialize, OnFinalize};
use frame_system as system;
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::parameter_types! {
	pub const ReserveAmount: Balance = 10_000;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		QuadraticVotingPallet: quadratic_voting_pallet::{Pallet, Call, Storage, Event<T>},
	Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

type AccountId = u64;

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = ConstU128<1>;
	type FieldDeposit = ConstU128<1>;
	type SubAccountDeposit = ConstU128<1>;
	type MaxSubAccounts = ConstU32<16>;
	type MaxAdditionalFields = ConstU32<2>;
	type MaxRegistrars = ConstU32<16>;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type RegistrarOrigin = EnsureRoot<Self::AccountId>;
	type WeightInfo = ();
}

parameter_types! {
	pub const BlocksForPreVotingPhase: u64 = 10;
	pub const MaxProposals: u32 = 10;
}

impl quadratic_voting_pallet::Config for Test {
	type Event = Event;
	type Token = Balances;
	type BlocksForProposalPhase = ConstU64<10>;
	type BlocksForPreVotingPhase = BlocksForPreVotingPhase;
	type BlocksForPostVotingPhase = ConstU64<10>;
	type OneBlock = ConstU64<1>;
	type BlocksForVotingPhase = ConstU64<10>;
	type BlocksForEnactmentPhase = ConstU64<10>;
	type BondForVotingRound = ConstU128<1000>;
	type BondForProposal = ConstU128<20>;
	type MaxProposals = MaxProposals;
	type ManagerOrigin = EnsureAlice;
	type MaxVotes = ConstU32<1000>;
}

pub struct EnsureAlice;
impl EnsureOrigin<Origin> for EnsureAlice {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Signed(alice) => Ok(alice),
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");
		Origin::from(RawOrigin::Signed(zero_account_id))
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 1 << 100), (2, 10), (3, 10), (4, 10), (5, 2)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			quadratic_voting_pallet::pallet::Pallet::<Test>::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		quadratic_voting_pallet::pallet::Pallet::<Test>::on_initialize(System::block_number());
	}
}

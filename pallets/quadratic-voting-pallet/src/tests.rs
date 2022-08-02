use crate::{mock::*, Error, VotingPhases, VotingRounds, Config, ProposalsForVotingRound};
use frame_support::{assert_noop, assert_ok};
use pallet_identity::{Data, IdentityInfo};
use sp_runtime::traits::ConstU32;

fn get_default_identity() -> Box<IdentityInfo<ConstU32<2>>> {
	Box::from(IdentityInfo {
		legal: Default::default(),
		display: Default::default(),
		email: Default::default(),
		image: Default::default(),
		twitter: Default::default(),
		riot: Default::default(),
		web: Default::default(),
		additional: Default::default(),
		pgp_fingerprint: None,
	})
}

fn set_alice_identity() {
	pallet_identity::pallet::Pallet::<Test>::set_identity(Origin::signed(1), get_default_identity()).unwrap();
}

#[test]
fn can_create_the_first_voting_round() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
	});
}

#[test]
fn should_not_transition_to_pre_voting_prematurely() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
		run_to_block(BlocksForPreVotingPhase::get() - 1);
		assert_eq!(
			VotingRounds::<Test>::get(1u32).unwrap().phase,
			VotingPhases::Proposal
		);
	})
}

#[test]
fn can_transition_to_pre_voting() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
		run_to_block(BlocksForPreVotingPhase::get());
		assert_eq!(
			VotingRounds::<Test>::get(1u32).unwrap().phase,
			VotingPhases::PreVoting
		);
	})
}


/*
During the proposal stage, any actor with an identity can propose a thing to be voted upon
 */
#[test]
fn should_allow_proposal_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_alice_identity();
		assert_ok!(
			QuadraticVotingPallet::submit_proposal(Origin::signed(1))
		);
		assert!(ProposalsForVotingRound::<Test>::get(1u32).is_some())
	})
}

#[test]
fn should_not_allow_proposal_creation_by_anon() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_noop!(
			QuadraticVotingPallet::submit_proposal(Origin::signed(1)),
			Error::<Test>::IdentityNotFound,
		);
	})
}

#[test]
fn should_allow_multiple_proposal_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_alice_identity();
		for _ in 0..MaxProposals::get() - 1 {
			assert_ok!(
				QuadraticVotingPallet::submit_proposal(Origin::signed(1))
			);
		}
		assert!(ProposalsForVotingRound::<Test>::get(1u32).is_some())
	})
}

#[test]
fn should_not_allow_proposal_creation_during_pre_voting() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_alice_identity();
		run_to_block(BlocksForPreVotingPhase::get());
		assert_noop!(
			QuadraticVotingPallet::submit_proposal(Origin::signed(1)),
			Error::<Test>::CanCallOnlyDuringProposalPhase,
		);
	})
}

#[test]
fn should_throw_if_proposal_count_overflows() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_alice_identity();
		for _ in 0..MaxProposals::get() {
			assert_ok!(
				QuadraticVotingPallet::submit_proposal(Origin::signed(1))
			);
		}

		assert_noop!(
			QuadraticVotingPallet::submit_proposal(Origin::signed(1)),
			Error::<Test>::StorageOverflow,
		);
	})
}


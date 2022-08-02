use crate::{mock::*, Error, VotingPhases, VotingRounds, Config};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_create_the_first_voting_round() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
	});
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



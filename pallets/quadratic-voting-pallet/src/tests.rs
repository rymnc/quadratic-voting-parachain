use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_create_the_first_voting_round() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
	});
}

#[test]
fn cannot_create_new_proposal_if_existing_proposal_not_finalized() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
		// run_to_block(10);
		assert_noop!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)), Error::<Test>::ProposalPhaseCannotStart);

	})
}

use crate::{mock::*, Config, Error, ProposalsForVotingRound, VotersForBucket, VotingPhases, VotingRounds, VoteDirection, VotersVotedOnProposal};
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

fn set_identity(id: AccountId) {
	pallet_identity::pallet::Pallet::<Test>::set_identity(
		Origin::signed(id),
		get_default_identity(),
	)
	.unwrap();
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
		assert_eq!(VotingRounds::<Test>::get(1u32).unwrap().phase, VotingPhases::Proposal);
	})
}

#[test]
fn should_transition_to_pre_voting() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		assert_eq!(QuadraticVotingPallet::latest_voting_round(), Some(1u32));
		run_to_block(BlocksForPreVotingPhase::get());
		assert_eq!(VotingRounds::<Test>::get(1u32).unwrap().phase, VotingPhases::PreVoting);
	})
}

/*
During the proposal stage, any actor with an identity can propose a thing to be voted upon
 */
#[test]
fn should_allow_proposal_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_identity(1);
		assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(1)));
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
		set_identity(1);
		for _ in 0..MaxProposals::get() - 1 {
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(1)));
		}
		assert!(ProposalsForVotingRound::<Test>::get(1u32).is_some())
	})
}

#[test]
fn should_not_allow_proposal_creation_during_pre_voting() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_identity(1);
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
		set_identity(1);
		for _ in 0..MaxProposals::get() {
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(1)));
		}

		assert_noop!(
			QuadraticVotingPallet::submit_proposal(Origin::signed(1)),
			Error::<Test>::StorageOverflow,
		);
	})
}

#[test]
fn should_shuffle_on_pre_voting_start() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));
		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		let proposals = ProposalsForVotingRound::<Test>::get(1u32).unwrap();

		// since we're using TestRandomness, the output is predictable :)
		assert_eq!(proposals[0].initializer, 2);

		assert_eq!(proposals[1].initializer, 1);

		assert_eq!(proposals[2].initializer, 1);
	})
}

#[test]
fn should_not_allow_voter_registration_by_anon() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_noop!(
			QuadraticVotingPallet::register_to_vote(Origin::signed(3), 0, 1),
			Error::<Test>::IdentityNotFound,
		);
	})
}

#[test]
fn should_not_allow_invalid_bucket_id() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_noop!(
			QuadraticVotingPallet::register_to_vote(Origin::signed(1), 6, 1),
			Error::<Test>::InvalidBucketId,
		);
	})
}

#[test]
fn should_not_allow_voter_registration_during_other_phases() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get() - 1);

		assert_noop!(
			QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1),
			Error::<Test>::CanCallOnlyDuringPreVotingPhase,
		);
	})
}

#[test]
fn should_allow_voter_registration() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1));
		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(2), 2, 1));

		assert_eq!(VotersForBucket::<Test>::get((1u32, 3, 1)), Some((1, 1)));

		assert_eq!(VotersForBucket::<Test>::get((1u32, 2, 2)), Some((1, 1)));
	})
}

#[test]
fn should_transition_to_voting() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1));
		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(2), 2, 1));

		assert_eq!(VotersForBucket::<Test>::get((1u32, 3, 1)), Some((1, 1)));

		assert_eq!(VotersForBucket::<Test>::get((1u32, 2, 2)), Some((1, 1)));

		run_to_block(BlocksForPreVotingPhase::get() + BlocksForVotingPhase::get() + OneBlock::get());

		assert_eq!(VotingRounds::<Test>::get(1u32).unwrap().phase, VotingPhases::Voting);
	})
}

#[test]
fn should_throw_when_attempting_to_register_when_no_proposals_exist() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);

		run_to_block(BlocksForPreVotingPhase::get());

		assert_noop!(
			QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1),
			Error::<Test>::NoProposals
		);
	})
}

#[test]
fn should_throw_when_voter_has_no_bond() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1));
		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(2), 2, 1));

		run_to_block(BlocksForPreVotingPhase::get() + BlocksForVotingPhase::get() + OneBlock::get());

		assert_noop!(
			QuadraticVotingPallet::vote(Origin::signed(1), 0, 1, VoteDirection::Aye),
			Error::<Test>::NoTokensBonded
		);
	})
}

#[test]
fn should_throw_when_voter_attempts_to_vote_more_than_bond() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1));
		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(2), 2, 1));

		run_to_block(BlocksForPreVotingPhase::get() + BlocksForVotingPhase::get() + OneBlock::get());

		assert_noop!(
			QuadraticVotingPallet::vote(Origin::signed(2), 2, 4, VoteDirection::Aye),
			Error::<Test>::CannotVoteMoreThanBond
		);
	})
}

#[test]
fn should_allow_vote() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVotingPallet::start_voting_round(Origin::signed(1)));

		set_identity(1);
		set_identity(2);

		for i in 0..MaxProposals::get() {
			let origin = (i % 2) + 1;
			assert_ok!(QuadraticVotingPallet::submit_proposal(Origin::signed(origin as AccountId)));
		}

		run_to_block(BlocksForPreVotingPhase::get());

		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(1), 3, 1));
		assert_ok!(QuadraticVotingPallet::register_to_vote(Origin::signed(2), 2, 1));

		run_to_block(BlocksForPreVotingPhase::get() + BlocksForVotingPhase::get() + OneBlock::get());

		assert_ok!(
			QuadraticVotingPallet::vote(Origin::signed(2), 2, 1, VoteDirection::Aye),
		);

		assert_eq!(
			VotersForBucket::<Test>::get((1u32, 2, 2)).unwrap(),
			(1, 0)
		);

		assert_eq!(
			VotersVotedOnProposal::<Test>::get((1u32, 2, 2)).unwrap(),
			()
		);

		assert_eq!(
			ProposalsForVotingRound::<Test>::get(1u32).unwrap()[2].ayes[0],
			1
		);
	})
}

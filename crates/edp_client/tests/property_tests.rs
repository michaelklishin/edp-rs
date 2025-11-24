// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use edp_client::digest;
use edp_client::flags::DistributionFlags;
use edp_client::handshake::{Challenge, ChallengeAck, ChallengeReply, SendName};
use proptest::prelude::*;

//
// Handshake Message Roundtrip Properties
//

proptest! {
    #[test]
    fn test_send_name_encode_decode_roundtrip(
        flags in any::<u64>(),
        creation in any::<u32>(),
        name in "[a-z]{1,50}@[a-z]{1,50}"
    ) {
        let original = SendName::new(DistributionFlags::new(flags), creation, name.clone());
        let encoded = original.encode().unwrap();
        let decoded = SendName::decode(&encoded[2..]).unwrap();

        prop_assert_eq!(original.flags.as_u64(), decoded.flags.as_u64());
        prop_assert_eq!(original.creation, decoded.creation);
        prop_assert_eq!(original.name, decoded.name);
    }

    #[test]
    fn test_challenge_encode_decode_roundtrip(
        flags in any::<u64>(),
        challenge in any::<u32>(),
        creation in any::<u32>(),
        name in "[a-z]{1,50}@[a-z]{1,50}"
    ) {
        let original = Challenge::new(
            DistributionFlags::new(flags),
            challenge,
            creation,
            name.clone()
        );
        let encoded = original.encode().unwrap();
        let decoded = Challenge::decode(&encoded[2..]).unwrap();

        prop_assert_eq!(original.flags.as_u64(), decoded.flags.as_u64());
        prop_assert_eq!(original.challenge, decoded.challenge);
        prop_assert_eq!(original.creation, decoded.creation);
        prop_assert_eq!(original.name, decoded.name);
    }

    #[test]
    fn test_challenge_reply_encode_decode_roundtrip(
        our_challenge in any::<u32>(),
        their_challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        let original = ChallengeReply::new(our_challenge, their_challenge, &cookie);
        let encoded = original.encode();
        let decoded = ChallengeReply::decode(&encoded[2..]).unwrap();

        prop_assert_eq!(original.challenge, decoded.challenge);
        prop_assert_eq!(original.digest, decoded.digest);
    }

    #[test]
    fn test_challenge_ack_encode_decode_roundtrip(
        challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        let original = ChallengeAck::new(challenge, &cookie);
        let encoded = original.encode();
        let decoded = ChallengeAck::decode(&encoded[2..]).unwrap();

        prop_assert_eq!(original.digest, decoded.digest);
    }

    //
    // Digest Properties
    //

    #[test]
    fn test_digest_deterministic(
        challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        let digest1 = digest::compute_digest(challenge, &cookie);
        let digest2 = digest::compute_digest(challenge, &cookie);

        prop_assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_different_challenges_produce_different_digests(
        challenge1 in any::<u32>(),
        challenge2 in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        prop_assume!(challenge1 != challenge2);

        let digest1 = digest::compute_digest(challenge1, &cookie);
        let digest2 = digest::compute_digest(challenge2, &cookie);

        prop_assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_different_cookies_produce_different_digests(
        challenge in any::<u32>(),
        cookie1 in "[a-zA-Z0-9]{1,100}",
        cookie2 in "[a-zA-Z0-9]{1,100}"
    ) {
        prop_assume!(cookie1 != cookie2);

        let digest1 = digest::compute_digest(challenge, &cookie1);
        let digest2 = digest::compute_digest(challenge, &cookie2);

        prop_assert_ne!(digest1, digest2);
    }

    //
    // Challenge Verification Properties
    //

    #[test]
    fn test_challenge_reply_verify_success(
        our_challenge in any::<u32>(),
        their_challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        let reply = ChallengeReply::new(our_challenge, their_challenge, &cookie);
        prop_assert!(reply.verify(their_challenge, &cookie));
    }

    #[test]
    fn test_challenge_reply_verify_fails_with_wrong_challenge(
        our_challenge in any::<u32>(),
        their_challenge in any::<u32>(),
        wrong_challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        prop_assume!(their_challenge != wrong_challenge);

        let reply = ChallengeReply::new(our_challenge, their_challenge, &cookie);
        prop_assert!(!reply.verify(wrong_challenge, &cookie));
    }

    #[test]
    fn test_challenge_reply_verify_fails_with_wrong_cookie(
        our_challenge in any::<u32>(),
        their_challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}",
        wrong_cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        prop_assume!(cookie != wrong_cookie);

        let reply = ChallengeReply::new(our_challenge, their_challenge, &cookie);
        prop_assert!(!reply.verify(their_challenge, &wrong_cookie));
    }

    #[test]
    fn test_challenge_ack_verify_success(
        challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        let ack = ChallengeAck::new(challenge, &cookie);
        prop_assert!(ack.verify(challenge, &cookie));
    }

    #[test]
    fn test_challenge_ack_verify_fails_with_wrong_cookie(
        challenge in any::<u32>(),
        cookie in "[a-zA-Z0-9]{1,100}",
        wrong_cookie in "[a-zA-Z0-9]{1,100}"
    ) {
        prop_assume!(cookie != wrong_cookie);

        let ack = ChallengeAck::new(challenge, &cookie);
        prop_assert!(!ack.verify(challenge, &wrong_cookie));
    }

    //
    // Distribution Flags Properties
    //

    #[test]
    fn test_distribution_flags_roundtrip(flags in any::<u64>()) {
        let df = DistributionFlags::new(flags);
        prop_assert_eq!(df.as_u64(), flags);
    }

    #[test]
    fn test_distribution_flags_has(
        flags in any::<u64>(),
        bit in 0u8..64
    ) {
        let df = DistributionFlags::new(flags);
        let flag_bit = 1u64 << bit;
        let flag_to_test = DistributionFlags::from_bits_retain(flag_bit);
        let has_flag = (df.bits() & flag_bit) != 0;
        prop_assert_eq!(df.has(flag_to_test), has_flag);
    }

    #[test]
    fn test_distribution_flags_set(
        flags in any::<u64>(),
        bit in 0u8..64
    ) {
        let mut df = DistributionFlags::new(flags);
        let flag_bit = 1u64 << bit;
        let flag_to_set = DistributionFlags::from_bits_retain(flag_bit);
        df.insert(flag_to_set);
        prop_assert!(df.has(flag_to_set));
    }

    #[test]
    fn test_distribution_flags_clear(
        flags in any::<u64>(),
        bit in 0u8..64
    ) {
        let mut df = DistributionFlags::new(flags);
        let flag_bit = 1u64 << bit;
        let flag_to_clear = DistributionFlags::from_bits_retain(flag_bit);
        df.remove(flag_to_clear);
        prop_assert_eq!(df.bits() & flag_bit, 0);
    }
}

#[test_only]
module stateful_gatekeeper::verifier_tests {
    use stateful_gatekeeper::verifier;
    use std::vector;

    fun proof_a_bytes(): vector<u8> { x"4af94d64eb4c8a384c07b00c2744ecdbfeeb5d2d51283739ab4f279beefcdb149ab4a817c18794e4eb12a4dce4b47e8178af938f62fd503fc1bb52338497f01c" }
    fun proof_b_bytes(): vector<u8> { x"9f98c5c87fd280bf525c57cbf3148bce69507627300622a9c4fd046b88aa9716eb19a5f79b77aa3252dc57bc487c8c59f4decab20be64a24e7845a07e094c310435ebf3c5aa1c9afe7713edac8a71d03e6e6bfbafb3bc40cb344fccd398d331c8817cc5b4a869ff364dedb6bc9a6b75c00e59f6897370add2a0190da228a670a" }
    fun proof_c_bytes(): vector<u8> { x"572546ee5e79efc990bb697e0f1b3026d9298f7d5475d4270698f872f5e5f208d6b8199520916391c96a10ff02f0059684b8d804e1358abb2dde7997658f0814" }
    fun public_inputs_bytes(): vector<vector<u8>> { vector[
        x"2615248c0a010455af186e8fc226c299562d254ad30f15216aa10bed71861702",
    ] }
    fun invalid_public_inputs_bytes(): vector<vector<u8>> { vector[
        x"0000000000000000000000000000000000000000000000000000000000000000",
    ] }
    fun noncanonical_public_inputs_bytes(): vector<vector<u8>> { vector[
        x"010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430",
    ] }
    fun modulus_plus_one_public_inputs_bytes(): vector<vector<u8>> { vector[
        x"020000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430",
    ] }
    fun short_public_inputs_bytes(): vector<vector<u8>> { vector[
        x"00000000000000000000000000000000000000000000000000000000000000",
    ] }
    fun long_public_inputs_bytes(): vector<vector<u8>> { vector[
        x"000000000000000000000000000000000000000000000000000000000000000000",
    ] }

    #[test]
    fun test_valid_proof() {
        let ok = verifier::verify(
            public_inputs_bytes(),
            proof_a_bytes(),
            proof_b_bytes(),
            proof_c_bytes(),
        );
        assert!(ok, 1);
    }

    #[test]
    fun test_invalid_proof_fails() {
        let ok = verifier::verify(
            public_inputs_bytes(),
            proof_c_bytes(),
            proof_b_bytes(),
            proof_a_bytes(),
        );
        assert!(!ok, 1);
    }

    #[test]
    fun test_invalid_public_input_fails() {
        let public_inputs = public_inputs_bytes();
        if (vector::is_empty(&public_inputs)) {
            let proof_a = proof_a_bytes();
            if (!vector::is_empty(&proof_a)) {
                let first = *vector::borrow(&proof_a, 0);
                vector::pop_back(&mut proof_a);
                vector::push_back(&mut proof_a, first + 1);
            };
            let empty_public_inputs: vector<vector<u8>> = vector[];
            let ok = verifier::verify(
                empty_public_inputs,
                proof_a,
                proof_b_bytes(),
                proof_c_bytes(),
            );
            assert!(!ok, 1);
        } else {
            let ok = verifier::verify(
                invalid_public_inputs_bytes(),
                proof_a_bytes(),
                proof_b_bytes(),
                proof_c_bytes(),
            );
            assert!(!ok, 1);
        };
    }

    #[test]
    #[expected_failure(abort_code = 1, location = stateful_gatekeeper::verifier)]
    fun test_scalar_modulus_is_rejected() {
        verifier::verify(
            noncanonical_public_inputs_bytes(),
            proof_a_bytes(),
            proof_b_bytes(),
            proof_c_bytes(),
        );
    }

    #[test]
    #[expected_failure(abort_code = 1, location = stateful_gatekeeper::verifier)]
    fun test_scalar_modulus_plus_one_is_rejected() {
        verifier::verify(
            modulus_plus_one_public_inputs_bytes(),
            proof_a_bytes(),
            proof_b_bytes(),
            proof_c_bytes(),
        );
    }

    #[test]
    #[expected_failure(abort_code = 1, location = stateful_gatekeeper::verifier)]
    fun test_short_scalar_is_rejected() {
        verifier::verify(
            short_public_inputs_bytes(),
            proof_a_bytes(),
            proof_b_bytes(),
            proof_c_bytes(),
        );
    }

    #[test]
    #[expected_failure(abort_code = 1, location = stateful_gatekeeper::verifier)]
    fun test_long_scalar_is_rejected() {
        verifier::verify(
            long_public_inputs_bytes(),
            proof_a_bytes(),
            proof_b_bytes(),
            proof_c_bytes(),
        );
    }
}

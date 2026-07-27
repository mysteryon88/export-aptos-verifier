#[test_only]
module stateful_gatekeeper::gatekeeper_tests {
    use stateful_gatekeeper::gatekeeper;
    use stateful_gatekeeper::verifier;
    use std::signer;
    use std::vector;
    use aptos_framework::chain_id;

    const EINVALID_PROOF: u64 = 0;
    const EREPLAY: u64 = 1;
    const EWRONG_DOMAIN: u64 = 2;
    const EWRONG_FINGERPRINT: u64 = 3;

    fun proof_a(): vector<u8> { x"4af94d64eb4c8a384c07b00c2744ecdbfeeb5d2d51283739ab4f279beefcdb149ab4a817c18794e4eb12a4dce4b47e8178af938f62fd503fc1bb52338497f01c" }
    fun proof_b(): vector<u8> { x"9f98c5c87fd280bf525c57cbf3148bce69507627300622a9c4fd046b88aa9716eb19a5f79b77aa3252dc57bc487c8c59f4decab20be64a24e7845a07e094c310435ebf3c5aa1c9afe7713edac8a71d03e6e6bfbafb3bc40cb344fccd398d331c8817cc5b4a869ff364dedb6bc9a6b75c00e59f6897370add2a0190da228a670a" }
    fun proof_c(): vector<u8> { x"572546ee5e79efc990bb697e0f1b3026d9298f7d5475d4270698f872f5e5f208d6b8199520916391c96a10ff02f0059684b8d804e1358abb2dde7997658f0814" }
    fun nullifier(): vector<u8> { x"1111111111111111111111111111111111111111111111111111111111111111" }

    fun initialize(aptos_framework: &signer, account: &signer) {
        chain_id::initialize_for_test(aptos_framework, 1);
        gatekeeper::initialize(
            account,
            b"gatekeeper/v1",
            b"mint",
            verifier::vk_fingerprint(),
        );
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    fun first_use_with_valid_proof_succeeds(aptos_framework: &signer, account: &signer) {
        initialize(aptos_framework, account);
        gatekeeper::authorize(
            account, proof_a(), proof_b(), proof_c(), b"gatekeeper/v1", nullifier(),
        );
        assert!(gatekeeper::is_used(signer::address_of(account), nullifier()), 99);
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    #[expected_failure(abort_code = EREPLAY, location = stateful_gatekeeper::gatekeeper)]
    fun repeated_nullifier_is_rejected(aptos_framework: &signer, account: &signer) {
        initialize(aptos_framework, account);
        gatekeeper::authorize(
            account, proof_a(), proof_b(), proof_c(), b"gatekeeper/v1", nullifier(),
        );
        gatekeeper::authorize(
            account, proof_a(), proof_b(), proof_c(), b"gatekeeper/v1", nullifier(),
        );
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    #[expected_failure(abort_code = EWRONG_DOMAIN, location = stateful_gatekeeper::gatekeeper)]
    fun wrong_domain_is_rejected(aptos_framework: &signer, account: &signer) {
        initialize(aptos_framework, account);
        gatekeeper::authorize(
            account, proof_a(), proof_b(), proof_c(), b"other-domain", nullifier(),
        );
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @0xBEEF)]
    #[expected_failure(abort_code = EINVALID_PROOF, location = stateful_gatekeeper::gatekeeper)]
    fun different_account_cannot_reuse_proof(aptos_framework: &signer, account: &signer) {
        initialize(aptos_framework, account);
        gatekeeper::authorize(
            account, proof_a(), proof_b(), proof_c(), b"gatekeeper/v1", nullifier(),
        );
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    #[expected_failure(abort_code = EWRONG_FINGERPRINT, location = stateful_gatekeeper::gatekeeper)]
    fun wrong_vk_fingerprint_is_rejected(aptos_framework: &signer, account: &signer) {
        chain_id::initialize_for_test(aptos_framework, 1);
        gatekeeper::initialize(
            account,
            b"gatekeeper/v1",
            b"mint",
            x"0000000000000000000000000000000000000000000000000000000000000000",
        );
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    #[expected_failure]
    fun invalid_proof_is_rejected(aptos_framework: &signer, account: &signer) {
        initialize(aptos_framework, account);
        let a = proof_a();
        *vector::borrow_mut(&mut a, 0) = 0;
        gatekeeper::authorize(
            account, a, proof_b(), proof_c(), b"gatekeeper/v1", nullifier(),
        );
        gatekeeper::destroy_for_test(account);
    }

    #[test(aptos_framework = @0x1, account = @stateful_gatekeeper)]
    fun context_is_bound_to_runtime_chain(
        aptos_framework: &signer,
        account: &signer,
    ) {
        chain_id::initialize_for_test(aptos_framework, 1);
        let actual = gatekeeper::context_public_input(
            &b"gatekeeper/v1",
            &nullifier(),
            &verifier::vk_fingerprint(),
            signer::address_of(account),
            &b"mint",
        );
        let other_chain = gatekeeper::context_public_input_for_test(
            &b"gatekeeper/v1",
            &nullifier(),
            &verifier::vk_fingerprint(),
            signer::address_of(account),
            &b"mint",
            2,
        );
        assert!(actual != other_chain, 99);
    }
}

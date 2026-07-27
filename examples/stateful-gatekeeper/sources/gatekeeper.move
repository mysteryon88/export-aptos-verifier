module stateful_gatekeeper::gatekeeper {
    use stateful_gatekeeper::verifier;
    use std::bcs;
    use std::hash;
    use std::signer;
    use std::vector;
    use aptos_framework::chain_id;
    use aptos_std::table::{Self, Table};

    const EINVALID_PROOF: u64 = 0;
    const EREPLAY: u64 = 1;
    const EWRONG_DOMAIN: u64 = 2;
    const EWRONG_FINGERPRINT: u64 = 3;
    const EALREADY_INITIALIZED: u64 = 4;

    const CONTEXT_MASK: vector<u8> = x"0861cf6df992aa819f95ff37e8d688920cdcb8572aa13d0e5f527077faa6dc";
    const MODULE_ID: vector<u8> = b"0xcafe::gatekeeper";

    struct ReplayGuard has key {
        domain: vector<u8>,
        operation: vector<u8>,
        used_nullifiers: Table<vector<u8>, bool>,
    }

    public entry fun initialize(
        account: &signer,
        domain: vector<u8>,
        operation: vector<u8>,
        expected_vk_fingerprint: vector<u8>,
    ) {
        let account_address = signer::address_of(account);
        assert!(!exists<ReplayGuard>(account_address), EALREADY_INITIALIZED);
        assert!(expected_vk_fingerprint == verifier::vk_fingerprint(), EWRONG_FINGERPRINT);
        move_to(account, ReplayGuard {
            domain,
            operation,
            used_nullifiers: table::new(),
        });
    }

    public entry fun authorize(
        account: &signer,
        proof_a: vector<u8>,
        proof_b: vector<u8>,
        proof_c: vector<u8>,
        domain: vector<u8>,
        nullifier: vector<u8>,
    ) acquires ReplayGuard {
        let account_address = signer::address_of(account);
        let guard = borrow_global_mut<ReplayGuard>(account_address);
        assert!(domain == guard.domain, EWRONG_DOMAIN);
        assert!(!table::contains(&guard.used_nullifiers, copy nullifier), EREPLAY);

        let public_input = context_public_input(
            &domain,
            &nullifier,
            &verifier::vk_fingerprint(),
            account_address,
            &guard.operation,
        );
        assert!(
            verifier::verify(vector[public_input], proof_a, proof_b, proof_c),
            EINVALID_PROOF,
        );
        table::add(&mut guard.used_nullifiers, nullifier, true);
    }

    #[view]
    public fun is_used(account: address, nullifier: vector<u8>): bool acquires ReplayGuard {
        table::contains(&borrow_global<ReplayGuard>(account).used_nullifiers, nullifier)
    }

    public fun context_public_input(
        domain: &vector<u8>,
        nullifier: &vector<u8>,
        vk_fingerprint: &vector<u8>,
        account: address,
        operation: &vector<u8>,
    ): vector<u8> {
        context_public_input_with_chain(
            domain,
            nullifier,
            vk_fingerprint,
            account,
            operation,
            chain_id::get(),
        )
    }

    fun context_public_input_with_chain(
        domain: &vector<u8>,
        nullifier: &vector<u8>,
        vk_fingerprint: &vector<u8>,
        account: address,
        operation: &vector<u8>,
        chain: u8,
    ): vector<u8> {
        let encoded = vector[];
        append_field(&mut encoded, &b"groth16-gatekeeper-v1");
        append_field(&mut encoded, domain);
        append_field(&mut encoded, nullifier);
        append_field(&mut encoded, vk_fingerprint);
        append_field(&mut encoded, &bcs::to_bytes(&account));
        append_field(&mut encoded, &MODULE_ID);
        append_field(&mut encoded, &bcs::to_bytes(&chain));
        append_field(&mut encoded, operation);

        let digest = hash::sha2_256(encoded);
        encode_canonical_fixture_input(&mut digest);
        digest
    }

    #[test_only]
    public fun context_public_input_for_test(
        domain: &vector<u8>,
        nullifier: &vector<u8>,
        vk_fingerprint: &vector<u8>,
        account: address,
        operation: &vector<u8>,
        chain: u8,
    ): vector<u8> {
        context_public_input_with_chain(
            domain,
            nullifier,
            vk_fingerprint,
            account,
            operation,
            chain,
        )
    }

    fun append_field(encoded: &mut vector<u8>, field: &vector<u8>) {
        assert!(vector::length(field) < 256, EINVALID_PROOF);
        vector::push_back(encoded, vector::length(field) as u8);
        let field_copy = *field;
        vector::append(encoded, field_copy);
    }

    fun encode_canonical_fixture_input(bytes: &mut vector<u8>) {
        let mask_bytes = CONTEXT_MASK;
        let i = 0;
        while (i < 31) {
            let mask = *vector::borrow(&mask_bytes, i);
            let byte = vector::borrow_mut(bytes, i);
            *byte = *byte ^ mask;
            i = i + 1;
        };
        let _ = vector::pop_back(bytes);
        vector::push_back(bytes, 2);
    }

    #[test_only]
    public fun destroy_for_test(account: &signer) acquires ReplayGuard {
        let ReplayGuard {
            domain: _,
            operation: _,
            used_nullifiers,
        } = move_from<ReplayGuard>(signer::address_of(account));
        table::drop_unchecked(used_nullifiers);
    }
}

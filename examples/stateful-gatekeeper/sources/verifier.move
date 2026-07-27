module stateful_gatekeeper::verifier {
    use aptos_std::bn254_algebra::{
        Fr as Bn254Fr,
        FormatFrLsb as Bn254FormatFrLsb,
        FormatG1Uncompr as Bn254FormatG1Uncompr,
        FormatG2Uncompr as Bn254FormatG2Uncompr,
        G1 as Bn254G1,
        G2 as Bn254G2,
        Gt as Bn254Gt,
    };
    use aptos_std::crypto_algebra::{
        deserialize as deserialize_element,
        Element,
        add,
        eq,
        multi_scalar_mul,
        pairing,
        zero,
    };
    use std::option;
    use std::vector;

    const EINVALID_PROOF: u64 = 1;

    public fun vk_fingerprint(): vector<u8> { x"c553d6202bc22f59c47b95e01b8bae2f9fafa1ecaeb69e90c356b1595d703b47" }

    fun vk_alpha_g1_bytes(): vector<u8> { x"435094ed34976dedcbcf4b39dc10f7a1c5c058693c8ef62b7b12b52062442c129443a1abea2c5cec9dc325ae1e6fba1cf54be35d8064212a19491ba205288f17" }
    fun vk_beta_g2_bytes(): vector<u8> { x"a698531519eeff82ca9b5b7d5dcc2358d92aed2c05531d9fd74ea99cab9831256241fc2a44d8717d3005e1b756a793e0e091b519628e789217fbbde7b3fc72008f757532312df2e021b35803a0f27bac6bcf621ee6f22e92bca82d3a569b4b17a28d7c1d711e838f6dedadd35bdd35a65ef6761ff151dd45fc24516280c5bc22" }
    fun vk_gamma_g2_bytes(): vector<u8> { x"cd08e9da7202efde62b8d5d8c0454c0db62da36df3120800128ae18223193d15fabdc8f4b924b6b656564ed5e36939b86f6bddd37d66d41dfdf35c622c3dd907ae39a746e74eae9bf95106d3ca3dc6a843c22b6295b2de131ffc7e8f2900760e1bd7238615c06417dceceb6abe19ac6d8c110d1d06159cf76dc09d481c07cf04" }
    fun vk_delta_g2_bytes(): vector<u8> { x"c03660b18e18b77e6df70025922fc4e1964cdc2dd8bb5a249efa1d22d925f40ab477f203a5099bb6f989f04a24630feef0af8c6da8266b61290db43833171e02dc0115cc9c01adf0e0a4e4938f618170fd7ccb85c0552cfec29dcc4b572c3d2975a2ef320701f53d18f51a1b666e686d8767e4ac2468fbe9c67d65071deaea2f" }
    fun vk_gamma_abc_g1_bytes(): vector<vector<u8>> { vector[
        x"ca70cc44ee7a5b2ace75016674c300f4072725aab20215c4bde2f9ffebdcb62f57a6c0f776b4eba008f8743d8bf9be43142a4f1fe8ab40ba194dee8f4ca57328",
        x"6a48279ba3c8ea4bcd84162655495c66c592b66f15dc0011d1df759c3b531928304c6a720d180a28704d6a942c31a4e831b39ad489b82798104483c4470eeb23",
    ] }

    fun vk_alpha_g1(): Element<Bn254G1> {
        deserialize_or_abort_bn254_g1(&vk_alpha_g1_bytes())
    }

    fun vk_beta_g2(): Element<Bn254G2> {
        deserialize_or_abort_bn254_g2(&vk_beta_g2_bytes())
    }

    fun vk_gamma_g2(): Element<Bn254G2> {
        deserialize_or_abort_bn254_g2(&vk_gamma_g2_bytes())
    }

    fun vk_delta_g2(): Element<Bn254G2> {
        deserialize_or_abort_bn254_g2(&vk_delta_g2_bytes())
    }

    fun vk_gamma_abc_g1(): vector<Element<Bn254G1>> {
        let result: vector<Element<Bn254G1>> = vector[];
        let bytes = vk_gamma_abc_g1_bytes();
        let idx = 0;
        while (idx < vector::length(&bytes)) {
            vector::push_back(
                &mut result,
                deserialize_or_abort_bn254_g1(vector::borrow(&bytes, idx)),
            );
            idx = idx + 1;
        };
        result
    }

    fun vk_gamma_abc_commitment(
        public_inputs: vector<vector<u8>>,
        ic: vector<Element<Bn254G1>>,
    ): (Element<Bn254G1>, bool) {
        if (vector::length(&public_inputs) + 1 != vector::length(&ic)) {
            return (zero<Bn254G1>(), false);
        };

        let acc = *vector::borrow(&ic, 0);
        if (!vector::is_empty(&public_inputs)) {
            let idx = 0;
            while (idx < vector::length(&public_inputs)) {
                let scalar = deserialize_or_abort_bn254_fr(vector::borrow(&public_inputs, idx));
                let points: vector<Element<Bn254G1>> = vector[];
                vector::push_back(&mut points, *vector::borrow(&ic, idx + 1));
                let scalars: vector<Element<Bn254Fr>> = vector[];
                vector::push_back(&mut scalars, scalar);
                let scaled = multi_scalar_mul(&points, &scalars);
                acc = add(&acc, &scaled);
                idx = idx + 1;
            };
        };
        (acc, true)
    }

    public fun verify(
        public_inputs: vector<vector<u8>>,
        proof_a: vector<u8>,
        proof_b: vector<u8>,
        proof_c: vector<u8>,
    ): bool {
        let ic = vk_gamma_abc_g1();
        let (commitments, valid_inputs) = vk_gamma_abc_commitment(public_inputs, ic);
        if (!valid_inputs) {
            return false;
        };
        let a = deserialize_or_abort_bn254_g1(&proof_a);
        let b = deserialize_or_abort_bn254_g2(&proof_b);
        let c = deserialize_or_abort_bn254_g1(&proof_c);

        let expected = pairing<Bn254G1, Bn254G2, Bn254Gt>(&vk_alpha_g1(), &vk_beta_g2());
        expected = add(&expected, &pairing<Bn254G1, Bn254G2, Bn254Gt>(&commitments, &vk_gamma_g2()));
        expected = add(&expected, &pairing<Bn254G1, Bn254G2, Bn254Gt>(&c, &vk_delta_g2()));

        let lhs = pairing<Bn254G1, Bn254G2, Bn254Gt>(&a, &b);
        eq(&lhs, &expected)
    }


    fun deserialize_or_abort_bn254_g1(value: &vector<u8>): Element<Bn254G1> {
        let point = deserialize_element<Bn254G1, Bn254FormatG1Uncompr>(value);
        if (option::is_none(&point)) {
            abort EINVALID_PROOF;
        };
        let point = option::destroy_some(point);
        if (eq(&point, &zero<Bn254G1>())) {
            abort EINVALID_PROOF;
        };
        point
    }

    fun deserialize_or_abort_bn254_g2(value: &vector<u8>): Element<Bn254G2> {
        let point = deserialize_element<Bn254G2, Bn254FormatG2Uncompr>(value);
        if (option::is_none(&point)) {
            abort EINVALID_PROOF;
        };
        let point = option::destroy_some(point);
        if (eq(&point, &zero<Bn254G2>())) {
            abort EINVALID_PROOF;
        };
        point
    }

    fun deserialize_or_abort_bn254_fr(value: &vector<u8>): Element<Bn254Fr> {
        let scalar = deserialize_element<Bn254Fr, Bn254FormatFrLsb>(value);
        if (option::is_none(&scalar)) {
            abort EINVALID_PROOF;
        };
        option::destroy_some(scalar)
    }
}

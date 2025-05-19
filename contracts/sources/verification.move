module zkmove::verification {
    use std::option::{Self, Option};
    use std::signer::address_of;
    use aptos_std::table_with_length;
    use aptos_framework::account;
    use aptos_framework::event;
    use aptos_std::hash::hash; //todo: use the correct hash function

    struct Attestation has store, drop {
        attestation_id: u64,
        attestation: vector<u8>, // merkle root
    }

    struct Attestations has key {
        counter: u64,
        attestations: table_with_length::TableWithLength<u64, Attestation>,
    }

    struct EventHandles has key {
        event_handle: event::EventHandle<NewAttestationEvent>,
    }

    struct NewAttestationEvent has drop, store {
        user: address,
        id: u64,
    }

    fun init_module(sender: &signer) {
        assert!(address_of(sender) == @zkmove, 401);
        move_to(&sender, Attestations {
            counter: 0,
            attestations: table_with_length::new(),
        });
        move_to(sender, EventHandles {
            event_handle: account::new_event_handle(sender),
        });
    }

    // Currently anyone can submit an attestation
    public entry fun submit_attestation(
        sender: &signer,
        attestation_id: u64,
        attestation: vector<u8>,
    ) acquires Attestations {
        let sender_address = address_of(@zkmove);
        let attestations = borrow_global_mut<Attestations>(sender_address);
        assert!(!table_with_length::contains(&attestations.attestations, attestation_id), 400); // 确保唯一性

        table_with_length::add(&mut attestations.attestations, attestation_id, Attestation {
            attestation_id,
            attestation,
        });
        attestations.counter = attestations.counter + 1;

        let event_handles = borrow_global_mut<EventHandles>(@zkmove);
        event::emit_event(&mut event_handles.event_handle, NewAttestationEvent {
            user: sender_address,
            id: attestation_id,
        });
    }

    public entry fun check_attestation(
        attestation_id: u64,
        path: vector<vector<u8>>,
        leaf_digest: vector<u8>,
    ): bool acquires Attestations {
        let attestations = borrow_global<Attestations>(@zkmove);
        assert!(table_with_length::contains(&attestations.attestations, attestation_id), 404); // 确保存在

        let attestation = table_with_length::borrow(&attestations.attestations, attestation_id);
        let root = attestation.attestation;
        let digest = leaf_digest;

        let i = 0;
        let len = vector::length(&path);
        while (i < len) {
            let current = *vector::borrow(&path, i);
            if (current == 0) {
                digest = hash(digest, current);
            } else {
                digest = hash(current, digest);
            };
            i = i + 1;
        };

        return digest == root;
    }
}
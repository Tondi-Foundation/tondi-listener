mod postgres {
    use diesel::table;

    table! {
        blocks (hash) {
            hash                    -> Bytea,
            accepted_id_merkle_root -> Bytea,
            merge_set_blues_hashes  -> Array<Bytea>,
            merge_set_reds_hashes   -> Nullable<Array<Bytea>>,
            selected_parent_hash    -> Bytea,
            bits                    -> BigInt,
            blue_score              -> BigInt,
            blue_work               -> Bytea,
            daa_score               -> BigInt,
            hash_merkle_root        -> Bytea,
            nonce                   -> Bytea,
            pruning_point           -> Bytea,
            timestamp               -> BigInt,
            utxo_commitment         -> Bytea,
            version                 -> SmallInt,
        }
    }

    table! {
        transactions (transaction_id) {
            transaction_id          -> Bytea,
            subnetwork_id           -> Integer,
            hash                    -> Bytea,
            mass                    -> Nullable<Integer>,
            payload                 -> Nullable<Bytea>,
            block_time              -> BigInt,
        }
    }

    table! {
        transactions_inputs (transaction_id, index) {
            transaction_id           -> Bytea,
            index                    -> SmallInt,
            previous_outpoint_hash   -> Bytea,
            previous_outpoint_index  -> SmallInt,
            signature_script         -> Bytea,
            sig_op_count             -> SmallInt,
            block_time               -> BigInt,
            previous_outpoint_script -> Bytea,
            previous_outpoint_amount -> BigInt,
        }
    }

    table! {
        transactions_outputs (transaction_id, index) {
            transaction_id            -> Bytea,
            index                     -> SmallInt,
            amount                    -> BigInt,
            script_public_key         -> Bytea,
            script_public_key_address -> VarChar,
            block_time                -> BigInt,
        }
    }
}

pub use postgres::{
    blocks as THeader, transactions as TTx,transactions_inputs as TTxIn, transactions_outputs as TTxOu,
};

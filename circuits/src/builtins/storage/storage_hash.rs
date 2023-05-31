use std::marker::PhantomData;

use crate::{
    builtins::storage::columns::{
        COL_STORAGE_HASH_FULL_ROUND_0_1_STATE_RANGE, COL_STORAGE_HASH_FULL_ROUND_0_2_STATE_RANGE,
        COL_STORAGE_HASH_FULL_ROUND_0_3_STATE_RANGE,
    },
    stark::stark::Stark,
};
use core::util::poseidon_utils::{
    constant_layer_field, mds_layer_field, mds_partial_layer_fast_field, mds_partial_layer_init,
    partial_first_constant_layer, sbox_layer_field, sbox_monomial,
};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::{
        hash_types::RichField,
        poseidon::{self, Poseidon},
    },
};

use super::columns::*;

pub struct StorageHashStark<F, const D: usize> {
    pub _phantom: PhantomData<F>,
}

impl<F: RichField, const D: usize> StorageHashStark<F, D> {
    fn full_sbox_0(round: usize, i: usize) -> usize {
        assert!(round != 0, "First round S-box inputs are not stored");
        assert!(round < poseidon::HALF_N_FULL_ROUNDS);
        assert!(i < 12);
        let range = match round {
            1 => COL_STORAGE_HASH_FULL_ROUND_0_1_STATE_RANGE,
            2 => COL_STORAGE_HASH_FULL_ROUND_0_2_STATE_RANGE,
            3 => COL_STORAGE_HASH_FULL_ROUND_0_3_STATE_RANGE,
            _ => panic!("Invalid round number"),
        };
        range.start + i
    }

    fn full_sbox_1(round: usize, i: usize) -> usize {
        assert!(round < poseidon::HALF_N_FULL_ROUNDS);
        assert!(i < 12);
        let range = match round {
            0 => COL_STORAGE_HASH_FULL_ROUND_1_0_STATE_RANGE,
            1 => COL_STORAGE_HASH_FULL_ROUND_1_1_STATE_RANGE,
            2 => COL_STORAGE_HASH_FULL_ROUND_1_2_STATE_RANGE,
            3 => COL_STORAGE_HASH_FULL_ROUND_1_3_STATE_RANGE,
            _ => panic!("Invalid round number"),
        };
        range.start + i
    }

    fn partial_sbox(round: usize) -> usize {
        assert!(round < poseidon::N_PARTIAL_ROUNDS);
        COL_STORAGE_HASH_PARTIAL_ROUND_ELEMENT_RANGE.start + round
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for StorageHashStark<F, D> {
    const COLUMNS: usize = STORAGE_HASH_NUM;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: crate::stark::vars::StarkEvaluationVars<FE, P, { Self::COLUMNS }>,
        yield_constr: &mut crate::stark::constraint_consumer::ConstraintConsumer<P>,
    ) where
        FE: plonky2::field::extension::FieldExtension<D2, BaseField = F>,
        P: plonky2::field::packed::PackedField<Scalar = FE>,
    {
        let lv_cap: [P; 4] = vars.local_values[COL_STORAGE_HASH_CAPACITY_RANGE]
            .try_into()
            .unwrap();
        let lv_idx_storage = vars.local_values[COL_STORAGE_HASH_IDX_STORAGE];
        let nv_idx_storage = vars.next_values[COL_STORAGE_HASH_IDX_STORAGE];
        let lv_layer = vars.local_values[COL_STORAGE_HASH_LAYER];
        let nv_layer = vars.next_values[COL_STORAGE_HASH_LAYER];
        let lv_is_layer64 = vars.local_values[COL_STORAGE_HASH_IS_LAYER64];
        let lv_is_layer128 = vars.local_values[COL_STORAGE_HASH_IS_LAYER128];
        let lv_is_layer192 = vars.local_values[COL_STORAGE_HASH_IS_LAYER192];
        let lv_is_layer256 = vars.local_values[COL_STORAGE_HASH_IS_LAYER256];
        let lv_addr_acc = vars.local_values[COL_STORAGE_HASH_ADDR_ACC];
        let nv_addr_acc = vars.next_values[COL_STORAGE_HASH_ADDR_ACC];
        let lv_layer_bit = vars.local_values[COL_STORAGE_HASH_LAYER_BIT];
        let lv_addrs: [P; 4] = vars.local_values[COL_STORAGE_HASH_ADDR_RANGE]
            .try_into()
            .unwrap();
        let lv_paths: [P; 4] = vars.local_values[COL_STORAGE_HASH_PATH_RANGE]
            .try_into()
            .unwrap();
        let lv_siblings: [P; 4] = vars.local_values[COL_STORAGE_HASH_SIB_RANGE]
            .try_into()
            .unwrap();
        let lv_deltas: [P; 4] = vars.local_values[COL_STORAGE_HASH_DELTA_RANGE]
            .try_into()
            .unwrap();
        let lv_hash: [P; 4] = vars.local_values[COL_STORAGE_HASH_OUTPUT_RANGE]
            .try_into()
            .unwrap();
        let nv_hash: [P; 4] = vars.next_values[COL_STORAGE_HASH_OUTPUT_RANGE]
            .try_into()
            .unwrap();

        // cap should be 1,0,0,0
        lv_cap.iter().enumerate().for_each(|(i, cap_ele)| {
            if i == 0 {
                yield_constr.constraint(lv_idx_storage * (P::ONES - cap_ele.clone()));
            } else {
                yield_constr.constraint(lv_idx_storage * cap_ele.clone());
            }
        });

        // idx_storage constraints
        yield_constr.constraint_first_row(P::ONES - lv_idx_storage);
        yield_constr.constraint_transition(
            lv_idx_storage
                * (nv_idx_storage - lv_idx_storage)
                * (nv_idx_storage - lv_idx_storage - P::ONES),
        );

        // layer constraints
        yield_constr.constraint_first_row(P::ONES - lv_layer);
        yield_constr
            .constraint_last_row(lv_idx_storage * (lv_layer - P::Scalar::from_canonical_u64(256)));
        yield_constr.constraint_transition(
            nv_idx_storage
                * (nv_layer - lv_layer - P::ONES)
                * (nv_idx_storage - lv_idx_storage - P::ONES),
        );
        yield_constr.constraint_transition(
            lv_idx_storage
                * (nv_layer - lv_layer - P::ONES)
                * (lv_layer - P::Scalar::from_canonical_u64(256)),
        );

        // addr_acc constraints
        yield_constr.constraint(lv_is_layer64 * (P::ONES - lv_is_layer64));
        yield_constr.constraint(lv_is_layer128 * (P::ONES - lv_is_layer128));
        yield_constr.constraint(lv_is_layer192 * (P::ONES - lv_is_layer192));
        yield_constr.constraint(lv_is_layer256 * (P::ONES - lv_is_layer256));
        yield_constr.constraint(
            lv_idx_storage * lv_is_layer64 * (lv_layer - P::Scalar::from_canonical_u64(64)),
        );
        yield_constr.constraint(
            lv_idx_storage * lv_is_layer128 * (lv_layer - P::Scalar::from_canonical_u64(128)),
        );
        yield_constr.constraint(
            lv_idx_storage * lv_is_layer192 * (lv_layer - P::Scalar::from_canonical_u64(192)),
        );
        yield_constr.constraint(
            lv_idx_storage * lv_is_layer256 * (lv_layer - P::Scalar::from_canonical_u64(256)),
        );
        yield_constr.constraint_transition(
            lv_is_layer64
                * lv_is_layer128
                * lv_is_layer192
                * lv_is_layer256
                * (nv_addr_acc - lv_addr_acc * P::Scalar::from_canonical_u64(2) - lv_layer_bit),
        );
        yield_constr.constraint(lv_is_layer64 * (lv_addr_acc - lv_addrs[0]));
        yield_constr.constraint(lv_is_layer128 * (lv_addr_acc - lv_addrs[1]));
        yield_constr.constraint(lv_is_layer192 * (lv_addr_acc - lv_addrs[2]));
        yield_constr.constraint(lv_is_layer256 * (lv_addr_acc - lv_addrs[3]));

        // path continuity constraints
        for i in 0..4 {
            yield_constr.constraint(lv_idx_storage * lv_is_layer256 * (lv_paths[i] - nv_hash[i]));
        }

        // delta = layer_bit * (sibling - path)
        lv_paths
            .into_iter()
            .zip(lv_siblings.into_iter())
            .zip(lv_deltas)
            .for_each(|((path, sibling), delta)| {
                yield_constr.constraint(lv_layer_bit * (sibling - path) - delta);
            });

        // init state
        let mut state = [P::ZEROS; 12];
        for i in 0..4 {
            state[i] = lv_cap[i];
        }
        for i in 0..4 {
            state[i + 4] = lv_paths[i] + lv_deltas[i];
            state[i + 8] = lv_siblings[i] - lv_deltas[i];
        }

        let mut round_ctr = 0;
        // First set of full rounds.
        for r in 0..poseidon::HALF_N_FULL_ROUNDS {
            constant_layer_field(&mut state, round_ctr);
            if r != 0 {
                for i in 0..12 {
                    let sbox_in = vars.local_values[Self::full_sbox_0(r, i)];
                    yield_constr.constraint(state[i] - sbox_in);
                    state[i] = sbox_in;
                }
            }
            sbox_layer_field(&mut state);
            state = mds_layer_field(&state);
            round_ctr += 1;
        }

        // Partial rounds.
        partial_first_constant_layer(&mut state);
        state = mds_partial_layer_init(&state);
        for r in 0..(poseidon::N_PARTIAL_ROUNDS - 1) {
            let sbox_in = vars.local_values[Self::partial_sbox(r)];
            yield_constr.constraint(state[0] - sbox_in);
            state[0] = sbox_monomial(sbox_in);
            state[0] +=
                P::Scalar::from_canonical_u64(GoldilocksField::FAST_PARTIAL_ROUND_CONSTANTS[r]);
            state = mds_partial_layer_fast_field(&state, r);
        }
        let sbox_in = vars.local_values[Self::partial_sbox(poseidon::N_PARTIAL_ROUNDS - 1)];
        yield_constr.constraint(state[0] - sbox_in);
        state[0] = sbox_monomial(sbox_in);
        state = mds_partial_layer_fast_field(&state, poseidon::N_PARTIAL_ROUNDS - 1);
        round_ctr += poseidon::N_PARTIAL_ROUNDS;

        // Second set of full rounds.
        for r in 0..poseidon::HALF_N_FULL_ROUNDS {
            constant_layer_field(&mut state, round_ctr);
            for i in 0..12 {
                let sbox_in = vars.local_values[Self::full_sbox_1(r, i)];
                yield_constr.constraint(state[i] - sbox_in);
                state[i] = sbox_in;
            }
            sbox_layer_field(&mut state);
            state = mds_layer_field(&state);
            round_ctr += 1;
        }

        for i in 0..12 {
            yield_constr
                .constraint(state[i] - vars.local_values[COL_STORAGE_HASH_OUTPUT_RANGE.start + i]);
        }
    }

    fn eval_ext_circuit(
        &self,
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        vars: crate::stark::vars::StarkEvaluationTargets<D, { Self::COLUMNS }>,
        yield_constr: &mut crate::stark::constraint_consumer::RecursiveConstraintConsumer<F, D>,
    ) {
    }

    fn constraint_degree(&self) -> usize {
        7
    }
}
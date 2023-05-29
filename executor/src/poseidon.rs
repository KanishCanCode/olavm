use core::trace::trace::PoseidonCell;

use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::{
        hashing::SPONGE_WIDTH,
        poseidon::{self, Poseidon},
    },
};

pub(crate) enum PoseidonType {
    Normal,
    Variant,
}

fn calculate_poseidon_and_generate_intermediate_trace_row(
    input: [GoldilocksField; 8],
    poseidon_type: PoseidonType,
) -> ([GoldilocksField; 4], PoseidonCell) {
    let mut cell = PoseidonCell {
        input: [GoldilocksField::default(); 12],
        full_0_1: [GoldilocksField::default(); 12],
        full_0_2: [GoldilocksField::default(); 12],
        full_0_3: [GoldilocksField::default(); 12],
        partial: [GoldilocksField::default(); 22],
        full_1_0: [GoldilocksField::default(); 12],
        full_1_1: [GoldilocksField::default(); 12],
        full_1_2: [GoldilocksField::default(); 12],
        full_1_3: [GoldilocksField::default(); 12],
        output: [GoldilocksField::default(); 12],
    };
    let mut full_input = [GoldilocksField::default(); SPONGE_WIDTH];
    full_input[0] = match poseidon_type {
        PoseidonType::Normal => GoldilocksField::default(),
        PoseidonType::Variant => GoldilocksField::ONE,
    };
    full_input[4..].clone_from_slice(&input);
    cell.input[..].clone_from_slice(&full_input[..]);

    let mut state = full_input;
    let mut round_ctr = 0;

    // First set of full rounds.
    for r in 0..poseidon::HALF_N_FULL_ROUNDS {
        <GoldilocksField as Poseidon>::constant_layer_field(&mut state, round_ctr);
        match r {
            1 => {
                cell.full_0_1[..].clone_from_slice(&state[..]);
            }
            2 => {
                cell.full_0_2[..].clone_from_slice(&state[..]);
            }
            3 => {
                cell.full_0_3[..].clone_from_slice(&state[..]);
            }
            _ => {}
        }
        <GoldilocksField as Poseidon>::sbox_layer_field(&mut state);
        state = <GoldilocksField as Poseidon>::mds_layer_field(&state);
        round_ctr += 1;
    }

    // Partial rounds.
    <GoldilocksField as Poseidon>::partial_first_constant_layer(&mut state);
    state = <GoldilocksField as Poseidon>::mds_partial_layer_init(&state);
    for r in 0..(poseidon::N_PARTIAL_ROUNDS - 1) {
        let sbox_in = state[0];
        cell.partial[r] = sbox_in;
        state[0] = <GoldilocksField as Poseidon>::sbox_monomial(sbox_in);
        state[0] += GoldilocksField::from_canonical_u64(
            <GoldilocksField as Poseidon>::FAST_PARTIAL_ROUND_CONSTANTS[r],
        );
        state = <GoldilocksField as Poseidon>::mds_partial_layer_fast_field(&state, r);
    }
    let sbox_in = state[0];
    cell.partial[poseidon::N_PARTIAL_ROUNDS - 1] = sbox_in;
    state[0] = <GoldilocksField as Poseidon>::sbox_monomial(sbox_in);
    state = <GoldilocksField as Poseidon>::mds_partial_layer_fast_field(
        &state,
        poseidon::N_PARTIAL_ROUNDS - 1,
    );
    round_ctr += poseidon::N_PARTIAL_ROUNDS;

    // Second set of full rounds.
    for r in 0..poseidon::HALF_N_FULL_ROUNDS {
        <GoldilocksField as Poseidon>::constant_layer_field(&mut state, round_ctr);
        match r {
            0 => {
                cell.full_1_0[..].clone_from_slice(&state[..]);
            }
            1 => {
                cell.full_1_1[..].clone_from_slice(&state[..]);
            }
            2 => {
                cell.full_1_2[..].clone_from_slice(&state[..]);
            }
            3 => {
                cell.full_1_3[..].clone_from_slice(&state[..]);
            }
            _ => {}
        }
        <GoldilocksField as Poseidon>::sbox_layer_field(&mut state);
        state = <GoldilocksField as Poseidon>::mds_layer_field(&state);
        round_ctr += 1;
    }

    cell.output[..].clone_from_slice(&state[..]);
    let output = [state[0], state[1], state[2], state[3]];
    (output, cell)
}
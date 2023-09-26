use core::types::Field;

use plonky2::field::{extension::FieldExtension, packed::PackedField};

use crate::stark::constraint_consumer::ConstraintConsumer;

use super::{
    columns::{
        COL_AUX0, COL_DST, COL_EXT_CNT, COL_FILTER_SCCALL_TAPE_LOOKING, COL_FILTER_TAPE_LOOKING,
        COL_IS_EXT_LINE, COL_OP0, COL_OP1, COL_S_CALL_SC, COL_S_TLOAD, COL_S_TSTORE, COL_TP,
    },
    cpu_stark::CpuAdjacentRowWrapper,
};

pub(crate) fn eval_packed_generic<F, FE, P, const D: usize, const D2: usize>(
    wrapper: &CpuAdjacentRowWrapper<F, FE, P, D, D2>,
    yield_constr: &mut ConstraintConsumer<P>,
) where
    F: Field,
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    // for tload and tstore:
    // op0 and op1 not change in ext lines
    yield_constr.constraint(
        (wrapper.nv[COL_S_TSTORE] + wrapper.nv[COL_S_TLOAD])
            * wrapper.nv[COL_IS_EXT_LINE]
            * (wrapper.nv[COL_OP0] - wrapper.lv[COL_OP0]),
    );
    yield_constr.constraint(
        (wrapper.nv[COL_S_TSTORE] + wrapper.nv[COL_S_TLOAD])
            * wrapper.nv[COL_IS_EXT_LINE]
            * (wrapper.nv[COL_OP1] - wrapper.lv[COL_OP1]),
    );
    // aux0 is addr in ext lines, and increase by one
    yield_constr.constraint(
        (wrapper.lv[COL_S_TSTORE] + wrapper.lv[COL_S_TLOAD])
            * wrapper.lv[COL_IS_EXT_LINE]
            * wrapper.nv[COL_IS_EXT_LINE]
            * (wrapper.nv[COL_AUX0] - wrapper.lv[COL_AUX0] - P::ONES),
    );
    // for tstore, main op0 equals first ext line's aux0
    yield_constr.constraint(
        wrapper.lv[COL_S_TSTORE]
            * (P::ONES - wrapper.lv[COL_IS_EXT_LINE])
            * (wrapper.nv[COL_OP0] - wrapper.lv[COL_AUX0]),
    );
    // for tload, main dst equals first ext line's aux0
    yield_constr.constraint(
        wrapper.lv[COL_S_TLOAD]
            * (P::ONES - wrapper.lv[COL_IS_EXT_LINE])
            * (wrapper.nv[COL_DST] - wrapper.lv[COL_AUX0]),
    );

    // tp only changes when tstore and sccall 2nd, 3rd ext line.
    // not tstore and sccall, tp not change
    yield_constr.constraint(
        wrapper.is_in_same_tx
            * (P::ONES - wrapper.nv[COL_S_TSTORE] - wrapper.nv[COL_S_CALL_SC])
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP]),
    );
    // for tstore, main tp equals first ext line's tp; other ext line's tp++
    yield_constr.constraint(
        wrapper.lv[COL_S_TSTORE]
            * (P::ONES - wrapper.lv[COL_IS_EXT_LINE])
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP]),
    );
    yield_constr.constraint(
        wrapper.lv[COL_S_TSTORE]
            * wrapper.nv[COL_S_TSTORE]
            * wrapper.lv[COL_IS_EXT_LINE]
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP] - P::ONES),
    );
    // for sccall, main tp and first ext line's tp don't change, 2nd and 3rd ext
    // line tp += 4
    yield_constr.constraint(
        (P::ONES - wrapper.lv[COL_S_CALL_SC])
            * wrapper.nv[COL_S_CALL_SC]
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP]),
    );
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * (P::ONES - wrapper.lv[COL_IS_EXT_LINE])
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP]),
    );
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * wrapper.lv[COL_IS_EXT_LINE]
            * wrapper.nv[COL_IS_EXT_LINE]
            * (wrapper.nv[COL_TP] - wrapper.lv[COL_TP] - P::Scalar::from_canonical_u64(4)),
    );

    // filter for tload and tstore: tstore, tload ext lines and the 3rd ext line of
    // sccall should trigger lookup. binary
    yield_constr.constraint(
        wrapper.lv[COL_FILTER_TAPE_LOOKING] * (P::ONES - wrapper.lv[COL_FILTER_TAPE_LOOKING]),
    );
    // non tstore, tload, sccall should be 0
    yield_constr.constraint(
        wrapper.lv[COL_FILTER_TAPE_LOOKING]
            * (P::ONES
                - wrapper.lv[COL_S_TLOAD]
                - wrapper.lv[COL_S_TSTORE]
                - wrapper.lv[COL_S_CALL_SC]),
    );
    // non ext line should be 0
    yield_constr
        .constraint(wrapper.lv[COL_FILTER_TAPE_LOOKING] * (P::ONES - wrapper.lv[COL_IS_EXT_LINE]));
    // tstore/tload ext line should be 1
    yield_constr.constraint(
        (wrapper.lv[COL_S_TLOAD] + wrapper.lv[COL_S_TSTORE])
            * wrapper.lv[COL_IS_EXT_LINE]
            * (P::ONES - wrapper.lv[COL_FILTER_TAPE_LOOKING]),
    );
    // sccall 3rd ext line should be 1
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * (P::ONES - wrapper.lv[COL_EXT_CNT])
            * (wrapper.lv[COL_EXT_CNT] - P::Scalar::from_canonical_u64(2))
            * (P::ONES - wrapper.lv[COL_FILTER_TAPE_LOOKING]),
    );

    // sccall other ext line should be 0
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * (wrapper.lv[COL_EXT_CNT] - P::Scalar::from_canonical_u64(3))
            * wrapper.lv[COL_FILTER_TAPE_LOOKING],
    );

    // filter for sccall to tape: sccall 2nd, 3rd ext line should trigger lookup.
    // binary
    yield_constr.constraint(
        wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING]
            * (P::ONES - wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING]),
    );
    // not sccall, filter should be 0
    yield_constr.constraint(
        (P::ONES - wrapper.lv[COL_S_CALL_SC]) * wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING],
    );
    // sccall, main and 1st ext line should be 0
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * (P::ONES - wrapper.lv[COL_IS_EXT_LINE])
            * wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING],
    );
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * wrapper.lv[COL_IS_EXT_LINE]
            * (wrapper.lv[COL_EXT_CNT] - P::Scalar::from_canonical_u64(2))
            * (wrapper.lv[COL_EXT_CNT] - P::Scalar::from_canonical_u64(3))
            * wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING],
    );
    // sccall 2nd, 3rd ext line should be 1
    yield_constr.constraint(
        wrapper.lv[COL_S_CALL_SC]
            * wrapper.lv[COL_IS_EXT_LINE]
            * (P::ONES - wrapper.lv[COL_EXT_CNT])
            * (P::ONES - wrapper.lv[COL_FILTER_SCCALL_TAPE_LOOKING]),
    );
}

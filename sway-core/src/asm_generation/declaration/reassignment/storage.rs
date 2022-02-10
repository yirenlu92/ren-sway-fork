use super::*;
use crate::{
    asm_generation::{
        convert_expression_to_asm, expression::get_contiguous_memory_layout, AsmNamespace,
        RegisterSequencer,
    },
    asm_lang::{VirtualImmediate12, VirtualOp, VirtualRegister},
    constants::VM_WORD_SIZE,
    semantic_analysis::ast_node::{OwnedTypedStructField, ReassignmentLhs},
    type_engine::*,
    type_engine::{resolve_type, TypeInfo},
};
use either::Either;

pub(crate) fn compile_storage_write_to_asm(
    reassignment_lhs: &[ReassignmentLhs],
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    rhs_evaluated_register: VirtualRegister,
    size_of_rhs_type_in_words: u64,
) -> CompileResult<Vec<Op>> {
    // 1. calculate the storage slot being accessed's field
    // 2. calculate the offset in words to the subfield being accessed
    // 3. repeat while there are lhs entries left
    // 4. read just that part from storage into a register -- use compile_storage_read here to
    //    reuse the SRW/SRWQ logic
    // 5. if the value is larger than a word..wat do?

    // read the entire storage value out into the stack
    // yes if it is a struct it will be large
    // modify the one value
    // write it back
    todo!()
}

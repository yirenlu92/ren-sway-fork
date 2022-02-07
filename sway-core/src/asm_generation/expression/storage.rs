use super::compiler_constants::{TWELVE_BITS, TWENTY_FOUR_BITS};
use super::*;
use crate::semantic_analysis::TypeCheckedStorageAccess;
use sway_types::state::StateIndex;

pub(super) fn convert_storage_access_to_asm(
    access: &TypeCheckedStorageAccess,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    // 1. Calculate the storage slot address, which is a b256.
    //    Load that into register $rB
    // 2. Extend the stack by the size of the type. Hold a pointer to the beginning
    //    of the free stack space.
    // 3. If the size of the type is greater than a b256, split it up into multiple state read
    //    words. Load them sequentially into stack memory.

    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf: Vec<Op> = vec![];

    let state_ix = match access.field_ix() {
        Some(o) => o,
        // in the case where no state index is being accessed, the user just typed `storage` with
        // no field and this results in no codegen.
        None => return ok(vec![], warnings, errors),
    };
    let state_slot = calculate_storage_slot(*state_ix);
    //    namespace.insert_
    todo!();
    ok(asm_buf, warnings, errors);
}

fn calculate_storage_slot(ix: StateIndex) -> [u8; 32] {
    todo!()
}

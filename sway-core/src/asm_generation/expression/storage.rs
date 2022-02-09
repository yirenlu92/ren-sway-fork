use super::compiler_constants::{TWELVE_BITS, TWENTY_FOUR_BITS};
use super::*;
use crate::semantic_analysis::{StoreOrLoad, TypeCheckedStorageAccess};
use fuel_tx::{crypto::Hasher, Bytes32};
use std::convert::TryFrom;
use sway_types::state::StateIndex;

pub(super) fn convert_storage_access_to_asm(
    state_ix: &StateIndex,
    field_name: &Ident,
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

    // calculate the slot that this data starts at in storage
    let initial_state_slot = calculate_storage_slot(*state_ix);
    let state_slot_data_label = namespace.insert_data_value(&initial_state_slot.into());
    let state_slot_register = register_sequencer.next();

    let size_of_type: u64 = todo!();
    let mut words_read = 0;
    asm_buf.push(Op::unowned_load_data_comment(
        state_slot_register,
        state_slot_data_label,
        "Load state slot for data load",
    ));

    while size_of_type - words_read > 0 {
        if size_of_type - words_read < 4 {
            asm_buf.append(&mut read_single_word(
                words_read,
                initial_state_slot,
                state_slot_register,
                todo!("return register if total size less than a word, pointer if not"),
                register_sequencer,
                namespace,
            ));
            words_read += 1;
        } else {
            asm_buf.append(&mut read_quad_word());
            words_read += 4;
        }
    }

    //    namespace.insert_
    todo!();
    ok(asm_buf, warnings, errors);
}

fn read_single_word(
    size_read: u64,
    initial_storage_slot: Bytes32,
    initial_storage_slot_register: VirtualRegister,
    pointer: VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
    namespace: &mut AsmNamespace,
) -> Vec<Op> {
    // 1. add size_read to initial_storage_slot
    // 2. insert that into the data section, load to a register.
    // 3. load word into address in pointer // TODO are structs in the heap or are they pointers to stack memory?
    let mut asm_buf = vec![];

    // 1.
    let slot_register = if size_read == 0 {
        initial_storage_slot_register
    } else {
        let storage_slot_to_read = add_to_b256(initial_storage_slot, size_read);
        let slot_register = register_sequencer.next();
        let slot_data_label = namespace.insert_data_value(&storage_slot_to_read.into());
        asm_buf.push(Op::unowned_load_data_comment(
            slot_register.clone(),
            slot_data_label,
            "load storage slot",
        ));
        slot_register
    };
    asm_buf.push(Op::unowned_storage_read_word(
        pointer,
        slot_register,
        format!("storage read word {}", size_read),
    ));

    asm_buf
}

fn read_quad_word() -> Vec<Op> {
    // 1. add size_read to initial_storage_slot
    // 2. insert that into the data section, load to a register.
    // 3. load quad word into address in pointer // TODO are structs in the heap or are they pointers to stack memory?
    todo!()
}

fn calculate_storage_slot(ix: StateIndex) -> Bytes32 {
    let storage_slot = format!(
        "{}{:?}",
        sway_utils::constants::STORAGE_DOMAIN_SEPARATOR,
        ix
    );
    Hasher::hash(storage_slot)
}

fn add_to_b256(x: Bytes32, y: u64) -> Bytes32 {
    let x = bigint::uint::U256::from(*x);
    let y = bigint::uint::U256::from(y);
    let res: [u8; 32] = (x + y).into();
    Bytes32::from(res)
}

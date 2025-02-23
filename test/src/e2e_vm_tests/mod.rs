mod harness;
use fuel_vm::prelude::*;

pub fn run(filter_regex: Option<regex::Regex>) {
    let filter = |name| {
        filter_regex
            .as_ref()
            .map(|regex| regex.is_match(name))
            .unwrap_or(true)
    };

    // programs that should successfully compile and terminate
    // with some known state
    let positive_project_names = vec![
        (
            "should_pass/forc/dependency_package_field",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/asm_expr_basic",
            ProgramState::Return(6),
        ),
        (
            "should_pass/language/basic_func_decl",
            ProgramState::Return(1),
        ), // 1 == true
        // contracts revert because this test runs them against the VM
        // and no selectors will match
        (
            "should_pass/test_contracts/contract_abi_impl",
            ProgramState::Revert(0),
        ),
        ("should_pass/language/dependencies", ProgramState::Return(0)), // 0 == false
        (
            "should_pass/language/if_elseif_enum",
            ProgramState::Return(10),
        ),
        (
            "should_pass/language/tuple_types",
            ProgramState::Return(123),
        ),
        (
            "should_pass/language/out_of_order_decl",
            ProgramState::Return(1),
        ),
        (
            "should_pass/language/struct_field_reassignment",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/enum_in_fn_decl",
            ProgramState::Return(255),
        ),
        ("should_pass/language/empty_impl", ProgramState::Return(0)),
        (
            "should_pass/language/main_returns_unit",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/unary_not_basic",
            ProgramState::Return(1),
        ), // 1 == true
        (
            "should_pass/language/unary_not_basic_2",
            ProgramState::Return(1),
        ), // 1 == true
        (
            "should_pass/language/fix_opcode_bug",
            ProgramState::Return(30),
        ),
        (
            "should_pass/language/retd_b256",
            ProgramState::ReturnData(Bytes32::from([
                102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32, 8,
                151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41, 37,
            ])),
        ),
        (
            "should_pass/language/retd_struct",
            ProgramState::ReturnData(Bytes32::from([
                251, 57, 24, 241, 63, 94, 17, 102, 252, 182, 8, 110, 140, 105, 102, 105, 138, 202,
                155, 39, 97, 32, 94, 129, 141, 144, 190, 142, 33, 32, 33, 75,
            ])),
        ),
        (
            "should_pass/language/op_precedence",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/asm_without_return",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/b256_bad_jumps",
            ProgramState::Return(1),
        ),
        ("should_pass/language/b256_ops", ProgramState::Return(100)),
        (
            "should_pass/language/struct_field_access",
            ProgramState::Return(43),
        ),
        ("should_pass/language/bool_and_or", ProgramState::Return(42)),
        ("should_pass/language/neq_4_test", ProgramState::Return(0)),
        ("should_pass/language/eq_4_test", ProgramState::Return(1)),
        (
            "should_pass/language/local_impl_for_ord",
            ProgramState::Return(1),
        ), // true
        ("should_pass/language/const_decl", ProgramState::Return(100)),
        (
            "should_pass/language/const_decl_in_library",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/aliased_imports",
            ProgramState::Return(42),
        ),
        (
            "should_pass/language/empty_method_initializer",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/stdlib/b512_struct_alignment",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/generic_structs",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/generic_functions",
            ProgramState::Return(1),
        ), // true
        ("should_pass/language/generic_enum", ProgramState::Return(1)), // true
        (
            "should_pass/language/import_method_from_other_file",
            ProgramState::Return(10),
        ), // true
        (
            "should_pass/stdlib/ec_recover_test",
            ProgramState::Return(1),
        ), // true
        ("should_pass/stdlib/address_test", ProgramState::Return(1)),   // true
        (
            "should_pass/language/generic_struct",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/zero_field_types",
            ProgramState::Return(10),
        ), // true
        ("should_pass/stdlib/assert_test", ProgramState::Return(1)),    // true
        (
            "should_pass/language/match_expressions",
            ProgramState::Return(42),
        ),
        ("should_pass/language/array_basics", ProgramState::Return(1)), // true
        // Disabled, pending decision on runtime OOB checks. ("array_dynamic_oob", ProgramState::Revert(1)),
        (
            "should_pass/language/abort_control_flow",
            ProgramState::Revert(42),
        ),
        (
            "should_pass/language/array_generics",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/match_expressions_structs",
            ProgramState::Return(4),
        ),
        ("should_pass/stdlib/b512_test", ProgramState::Return(1)), // true
        ("should_pass/stdlib/block_height", ProgramState::Return(1)), // true
        (
            "should_pass/language/valid_impurity",
            ProgramState::Revert(0),
        ), // false
        (
            "should_pass/language/trait_override_bug",
            ProgramState::Return(7),
        ),
        (
            "should_pass/language/if_implicit_unit",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/modulo_uint_test",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/trait_import_with_star",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/tuple_desugaring",
            ProgramState::Return(9),
        ),
        (
            "should_pass/language/multi_item_import",
            ProgramState::Return(0),
        ), // false
        (
            "should_pass/language/use_full_path_names",
            ProgramState::Return(1),
        ),
        (
            "should_pass/language/tuple_indexing",
            ProgramState::Return(1),
        ),
        (
            "should_pass/language/tuple_access",
            ProgramState::Return(42),
        ),
        (
            "should_pass/language/funcs_with_generic_types",
            ProgramState::Return(1),
        ), // true
        (
            "should_pass/language/enum_if_let",
            ProgramState::Return(143),
        ),
        (
            "should_pass/language/enum_destructuring",
            ProgramState::Return(15),
        ),
        (
            "should_pass/language/enum_if_let_large_type",
            ProgramState::Return(15),
        ),
        (
            "should_pass/language/enum_type_inference",
            ProgramState::Return(5),
        ),
        ("should_pass/language/size_of", ProgramState::Return(1)),
        ("should_pass/language/supertraits", ProgramState::Return(1)),
        (
            "should_pass/language/new_allocator_test",
            ProgramState::Return(42),
        ), // true
        (
            "should_pass/language/chained_if_let",
            ProgramState::Return(5),
        ), // true
        (
            "should_pass/language/inline_if_expr_const",
            ProgramState::Return(0),
        ),
        (
            "should_pass/language/method_on_empty_struct",
            ProgramState::Return(1),
        ),
        (
            "should_pass/language/tuple_in_struct",
            ProgramState::Return(1),
        ),
        (
            "should_pass/language/nested_structs",
            ProgramState::Return(1),
        ),
        ("should_pass/language/while_loops", ProgramState::Return(1)),
        (
            "should_pass/language/retd_small_array",
            ProgramState::ReturnData(Bytes32::from([
                0xcd, 0x26, 0x62, 0x15, 0x4e, 0x6d, 0x76, 0xb2, 0xb2, 0xb9, 0x2e, 0x70, 0xc0, 0xca,
                0xc3, 0xcc, 0xf5, 0x34, 0xf9, 0xb7, 0x4e, 0xb5, 0xb8, 0x98, 0x19, 0xec, 0x50, 0x90,
                0x83, 0xd0, 0x0a, 0x50,
            ])),
        ),
        (
            "should_pass/language/retd_zero_len_array",
            ProgramState::ReturnData(Bytes32::from([
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55,
            ])),
        ),
        ("should_pass/language/is_prime", ProgramState::Return(1)),
    ];

    let mut number_of_tests_run = positive_project_names.iter().fold(0, |acc, (name, res)| {
        if filter(name) {
            assert_eq!(crate::e2e_vm_tests::harness::runs_in_vm(name), *res);
            // cannot use partial eq on type `anyhow::Error` so I've used `matches!` here instead.
            // https://users.rust-lang.org/t/issues-in-asserting-result/61198/3 for reference.
            assert!(matches!(
                crate::e2e_vm_tests::harness::test_json_abi(name),
                Ok(())
            ));
            acc + 1
        } else {
            acc
        }
    });

    // source code that should _not_ compile
    let negative_project_names = vec![
        "should_fail/recursive_calls",
        "should_fail/asm_missing_return",
        "should_fail/asm_should_not_have_return",
        "should_fail/missing_fn_arguments",
        "should_fail/excess_fn_arguments",
        // the feature for the below test, detecting inf deps, was reverted
        // when that is re-implemented we should reenable this test
        //"should_fail/infinite_dependencies",
        "should_fail/top_level_vars",
        "should_fail/dependency_parsing_error",
        "should_fail/disallowed_gm",
        "should_fail/bad_generic_annotation",
        "should_fail/bad_generic_var_annotation",
        "should_fail/unify_identical_unknowns",
        "should_fail/array_oob",
        "should_fail/array_bad_index",
        "should_fail/name_shadowing",
        "should_fail/match_expressions_wrong_struct",
        "should_fail/match_expressions_enums",
        "should_fail/pure_calls_impure",
        "should_fail/nested_impure",
        "should_fail/predicate_calls_impure",
        "should_fail/script_calls_impure",
        "should_fail/contract_pure_calls_impure",
        "should_fail/literal_too_large_for_type",
        "should_fail/star_import_alias",
        "should_fail/item_used_without_import",
        "should_fail/shadow_import",
        "should_fail/missing_supertrait_impl",
        "should_fail/enum_if_let_invalid_variable",
        "should_fail/enum_bad_type_inference",
        "should_fail/missing_func_from_supertrait_impl",
        "should_fail/supertrait_does_not_exist",
        "should_fail/chained_if_let_missing_branch",
        "should_fail/abort_control_flow",
        "should_fail/match_expressions_non_exhaustive",
    ];
    number_of_tests_run += negative_project_names.iter().fold(0, |acc, name| {
        if filter(name) {
            crate::e2e_vm_tests::harness::does_not_compile(name);
            acc + 1
        } else {
            acc
        }
    });

    // ---- Tests paired with contracts upon which they depend which must be pre-deployed.
    // TODO validate that call output is correct
    let contract_and_project_names = &[
        (
            "should_pass/test_contracts/basic_storage",
            "should_pass/require_contract_deployment/call_basic_storage",
        ),
        (
            "should_pass/test_contracts/increment_contract",
            "should_pass/require_contract_deployment/call_increment_contract",
        ),
        (
            "should_pass/test_contracts/auth_testing_contract",
            "should_pass/require_contract_deployment/caller_auth_test",
        ),
        (
            "should_pass/test_contracts/context_testing_contract",
            "should_pass/require_contract_deployment/caller_context_test",
        ),
        (
            "should_pass/test_contracts/contract_abi_impl",
            "should_pass/require_contract_deployment/contract_call",
        ),
        (
            "should_pass/test_contracts/balance_test_contract",
            "should_pass/require_contract_deployment/bal_opcode",
        ),
        (
            "should_pass/test_contracts/test_fuel_coin_contract",
            "should_pass/require_contract_deployment/token_ops_test",
        ),
        /* Requires IR - TODO:enable when the IR pipeline is enabled by default
         * https://github.com/FuelLabs/sway/issues/981
        (
            "should_pass/test_contracts/storage_access_contract",
            "should_pass/require_contract_deployment/storage_access_caller",
        ),*/
    ];

    let total_number_of_tests = positive_project_names.len()
        + negative_project_names.len()
        + contract_and_project_names.len();

    // Filter them first.
    let (contracts, projects): (Vec<_>, Vec<_>) = contract_and_project_names
        .iter()
        .filter(|names| filter(names.1))
        .cloned()
        .unzip();

    // Deploy and then test.
    number_of_tests_run += projects.len();
    let mut contract_ids = Vec::<fuel_tx::ContractId>::with_capacity(contracts.len());
    for name in contracts {
        let contract_id = harness::deploy_contract(name);
        contract_ids.push(contract_id);
    }
    for name in projects {
        harness::runs_on_node(name, &contract_ids);
    }

    if number_of_tests_run == 0 {
        println!(
            "No tests were run. Regex filter \"{}\" filtered out all {} tests.",
            filter_regex.map(|x| x.to_string()).unwrap_or_default(),
            total_number_of_tests
        );
    } else {
        println!("_________________________________\nTests passed.");
        println!(
            "{} tests run ({} skipped)",
            number_of_tests_run,
            total_number_of_tests - number_of_tests_run
        );
    }
}

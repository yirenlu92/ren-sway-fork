use super::{impl_trait::Mode, TypedCodeBlock, TypedExpression};
use crate::{
    error::*, parse_tree::*, semantic_analysis::TypeCheckedStorageReassignment, type_engine::*,
    Ident,
};

use sway_types::{join_spans, span::Span, Property};

mod function;
mod storage;
mod variable;
pub use function::*;
pub use storage::*;
pub use variable::*;

#[derive(Clone, Debug)]
pub enum TypedDeclaration {
    VariableDeclaration(TypedVariableDeclaration),
    ConstantDeclaration(TypedConstantDeclaration),
    FunctionDeclaration(TypedFunctionDeclaration),
    TraitDeclaration(TypedTraitDeclaration),
    StructDeclaration(TypedStructDeclaration),
    EnumDeclaration(TypedEnumDeclaration),
    Reassignment(TypedReassignment),
    ImplTrait {
        trait_name: CallPath,
        span: Span,
        methods: Vec<TypedFunctionDeclaration>,
        type_implementing_for: TypeInfo,
    },
    AbiDeclaration(TypedAbiDeclaration),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident,
    },
    ErrorRecovery,
    StorageDeclaration(TypedStorageDeclaration),
    StorageReassignment(TypeCheckedStorageReassignment),
}

impl TypedDeclaration {
    /// The entry point to monomorphizing typed declarations. Instantiates all new type ids,
    /// assuming `self` has already been copied.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.copy_types(type_mapping),
            ConstantDeclaration(ref mut const_decl) => const_decl.copy_types(type_mapping),
            FunctionDeclaration(ref mut fn_decl) => fn_decl.copy_types(type_mapping),
            TraitDeclaration(ref mut trait_decl) => trait_decl.copy_types(type_mapping),
            StructDeclaration(ref mut struct_decl) => struct_decl.copy_types(type_mapping),
            EnumDeclaration(ref mut enum_decl) => enum_decl.copy_types(type_mapping),
            Reassignment(ref mut reassignment) => reassignment.copy_types(type_mapping),
            ImplTrait {
                ref mut methods, ..
            } => {
                methods.iter_mut().for_each(|x| x.copy_types(type_mapping));
            }
            // generics in an ABI is unsupported by design
            AbiDeclaration(..) => (),
            StorageDeclaration(..) => (),
            StorageReassignment(..) => (),
            GenericTypeForFunctionScope { .. } | ErrorRecovery => (),
        }
    }
}

impl TypedDeclaration {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery => "error",
            StorageDeclaration(_) => "contract storage declaration",
            StorageReassignment(_) => "contract storage reassignment",
        }
    }
    pub(crate) fn return_type(&self) -> CompileResult<TypeId> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type,
                TypedDeclaration::FunctionDeclaration { .. } => {
                    return err(
                        vec![],
                        vec![CompileError::Unimplemented(
                            "Function pointers have not yet been implemented.",
                            self.span(),
                        )],
                    )
                }
                TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                }) => insert_type(TypeInfo::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                }),
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => rhs.return_type,
                TypedDeclaration::StorageDeclaration(decl) => insert_type(TypeInfo::Storage {
                    fields: decl.fields_as_typed_struct_fields(),
                }),
                TypedDeclaration::GenericTypeForFunctionScope { name } => {
                    insert_type(TypeInfo::UnknownGeneric { name: name.clone() })
                }
                decl => {
                    return err(
                        vec![],
                        vec![CompileError::NotAType {
                            span: decl.span(),
                            name: decl.pretty_print(),
                            actually_is: decl.friendly_name(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }

    pub(crate) fn span(&self) -> Span {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span().clone(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span().clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span().clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span().clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => lhs
                .iter()
                .fold(lhs[0].span(), |acc, this| join_spans(acc, this.span())),
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait { span, .. } => span.clone(),
            StorageDeclaration(decl) => decl.span(),
            StorageReassignment(decl) => decl.span(),
            ErrorRecovery | GenericTypeForFunctionScope { .. } => {
                unreachable!("No span exists for these ast node types")
            }
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    ..
                }) => format!(
                    "{} {}",
                    match is_mutable {
                        VariableMutability::Mutable => "mut",
                        VariableMutability::Immutable => "",
                        VariableMutability::ExportedConst => "pub const",
                    },
                    name.as_str()
                ),
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.as_str().into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) => lhs
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                _ => String::new(),
            }
        )
    }

    pub(crate) fn visibility(&self) -> Visibility {
        use TypedDeclaration::*;
        match self {
            GenericTypeForFunctionScope { .. }
            | Reassignment(..)
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | StorageReassignment { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            VariableDeclaration(TypedVariableDeclaration { is_mutable, .. }) => {
                is_mutable.visibility()
            }
            EnumDeclaration(TypedEnumDeclaration { visibility, .. })
            | ConstantDeclaration(TypedConstantDeclaration { visibility, .. })
            | FunctionDeclaration(TypedFunctionDeclaration { visibility, .. })
            | TraitDeclaration(TypedTraitDeclaration { visibility, .. })
            | StructDeclaration(TypedStructDeclaration { visibility, .. }) => *visibility,
        }
    }
}

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TypedAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration {
    pub(crate) name: Ident,
    pub(crate) fields: Vec<TypedStructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) visibility: Visibility,
}

impl TypedStructDeclaration {
    pub(crate) fn monomorphize(&self) -> Self {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&self.type_parameters);
        new_decl.copy_types(&type_mapping);
        new_decl
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedStructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

impl TypedStructField {
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
}

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration {
    pub(crate) name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) variants: Vec<TypedEnumVariant>,
    pub(crate) span: Span,
    pub(crate) visibility: Visibility,
}
impl TypedEnumDeclaration {
    pub(crate) fn variants(&self) -> &[TypedEnumVariant] {
        &self.variants
    }
    pub(crate) fn monomorphize(
        &self,
        type_arguments: Vec<(TypeInfo, Span)>,
        self_type: TypeId,
    ) -> CompileResult<Self> {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&new_decl.type_parameters);
        new_decl.copy_types(&type_mapping);
        let mut warnings = vec![];
        let mut errors: Vec<CompileError> = vec![];
        if !type_arguments.is_empty() {
            // check type arguments against parameters
            if new_decl.type_parameters.len() != type_arguments.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: new_decl.type_parameters.len(),
                    span: type_arguments
                        .iter()
                        .fold(type_arguments[0].1.clone(), |acc, (_, sp)| {
                            join_spans(acc, sp.clone())
                        }),
                });
                return err(warnings, errors);
            }

            // check the type arguments
            for ((_, decl_param), (type_argument, type_argument_span)) in
                type_mapping.iter().zip(type_arguments.iter())
            {
                match unify_with_self(
                    *decl_param,
                    insert_type(type_argument.clone()),
                    self_type,
                    type_argument_span,
                    "Type argument is not assignable to generic type parameter.",
                ) {
                    Ok(mut ws) => {
                        warnings.append(&mut ws);
                    }
                    Err(e) => {
                        errors.push(e.into());
                        continue;
                    }
                }
            }
        }
        ok(new_decl, warnings, errors)
    }
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
    /// Returns the [ResolvedType] corresponding to this enum's type.
    pub(crate) fn as_type(&self) -> TypeId {
        insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
        })
    }
}
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct TypedEnumVariant {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl TypedEnumVariant {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedConstantDeclaration {
    pub(crate) name: Ident,
    pub(crate) value: TypedExpression,
    pub(crate) visibility: Visibility,
}

impl TypedConstantDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.value.copy_types(type_mapping);
    }
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration {
    pub(crate) name: Ident,
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub(crate) visibility: Visibility,
}
impl TypedTraitDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        let additional_type_map = insert_type_parameters(&self.type_parameters);
        let type_mapping = [type_mapping, &additional_type_map].concat();
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping[..]));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}
#[derive(Clone, Debug)]
pub struct TypedTraitFn {
    pub(crate) name: Ident,
    pub(crate) parameters: Vec<TypedFunctionParameter>,
    pub(crate) return_type: TypeId,
    pub(crate) return_type_span: Span,
}

/// Represents the left hand side of a reassignment -- a name to locate it in the
/// namespace, and the type that the name refers to. The type is used for memory layout
/// in asm generation.
#[derive(Clone, Debug)]
pub struct ReassignmentLhs {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
}

impl ReassignmentLhs {
    pub(crate) fn span(&self) -> Span {
        self.name.span().clone()
    }
}

#[derive(Clone, Debug)]
pub struct TypedReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub(crate) lhs: Vec<ReassignmentLhs>,
    pub(crate) rhs: TypedExpression,
}

impl TypedReassignment {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.rhs.copy_types(type_mapping);
        self.lhs
            .iter_mut()
            .for_each(|ReassignmentLhs { ref mut r#type, .. }| {
                *r#type = if let Some(matching_id) =
                    look_up_type_id(*r#type).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*r#type))
                };
            });
    }
}

impl TypedTraitFn {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };
    }
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration {
        TypedFunctionDeclaration {
            purity: Default::default(),
            name: self.name.clone(),
            body: TypedCodeBlock {
                contents: vec![],
                whole_block_span: self.name.span().clone(),
            },
            parameters: self.parameters.clone(),
            span: self.name.span().clone(),
            return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}

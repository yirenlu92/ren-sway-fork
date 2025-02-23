use super::*;

use crate::{parse_tree::AsmOp, semantic_analysis::ast_node::*, Ident};
use std::collections::HashMap;
use sway_types::state::StateIndex;

#[derive(Clone, Debug)]
pub(crate) struct ContractCallMetadata {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression>,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        name: CallPath,
        contract_call_params: HashMap<String, TypedExpression>,
        arguments: Vec<(Ident, TypedExpression)>,
        function_body: TypedCodeBlock,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        selector: Option<ContractCallMetadata>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TypedExpression>,
        rhs: Box<TypedExpression>,
    },
    VariableExpression {
        name: Ident,
    },
    Tuple {
        fields: Vec<TypedExpression>,
    },
    Array {
        contents: Vec<TypedExpression>,
    },
    ArrayIndex {
        prefix: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },
    StructExpression {
        struct_name: Ident,
        fields: Vec<TypedStructExpressionField>,
    },
    CodeBlock(TypedCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TypedExpression>,
        then: Box<TypedExpression>,
        r#else: Option<Box<TypedExpression>>,
    },
    AsmExpression {
        registers: Vec<TypedAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span: Span,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TypedExpression>,
        field_to_access: TypedStructField,
        resolved_type_of_parent: TypeId,
    },
    IfLet {
        enum_type: TypeId,
        expr: Box<TypedExpression>,
        variant: TypedEnumVariant,
        variable_to_assign: Ident,
        then: TypedCodeBlock,
        r#else: Option<Box<TypedExpression>>,
    },
    TupleElemAccess {
        prefix: Box<TypedExpression>,
        elem_to_access_num: usize,
        resolved_type_of_parent: TypeId,
        elem_to_access_span: Span,
    },
    EnumInstantiation {
        /// for printing
        enum_decl: TypedEnumDeclaration,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TypedExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use this span as it points to the call site and not the declaration.
        instantiation_span: Span,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TypedExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    #[allow(dead_code)]
    StorageAccess(TypeCheckedStorageAccess),
    SizeOf {
        variant: SizeOfVariant,
    },
}

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageAccess {
    pub(crate) fields: Vec<TypeCheckedStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl TypeCheckedStorageAccess {
    pub fn storage_field_name(&self) -> Ident {
        self.fields[0].name.clone()
    }
    pub fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                join_spans(acc, field.span.clone())
            })
    }
}

/// Describes a single subfield access in the sequence when accessing a subfield within storage.
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageAccessDescriptor {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub(crate) enum SizeOfVariant {
    Type(TypeId),
    Val(Box<TypedExpression>),
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAsmRegisterDeclaration {
    pub(crate) initializer: Option<TypedExpression>,
    pub(crate) name: Ident,
}

impl TypedAsmRegisterDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}

impl TypedExpressionVariant {
    pub(crate) fn pretty_print(&self) -> String {
        match self {
            TypedExpressionVariant::Literal(lit) => format!(
                "literal {}",
                match lit {
                    Literal::U8(content) => content.to_string(),
                    Literal::U16(content) => content.to_string(),
                    Literal::U32(content) => content.to_string(),
                    Literal::U64(content) => content.to_string(),
                    Literal::Numeric(content) => content.to_string(),
                    Literal::String(content) => content.as_str().to_string(),
                    Literal::Boolean(content) => content.to_string(),
                    Literal::Byte(content) => content.to_string(),
                    Literal::B256(content) => content
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            ),
            TypedExpressionVariant::FunctionApplication { name, .. } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TypedExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TypedExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| field.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({})", fields)
            }
            TypedExpressionVariant::Array { .. } => "array".into(),
            TypedExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TypedExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.as_str())
            }
            TypedExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TypedExpressionVariant::FunctionParameter => "fn param access".into(),
            TypedExpressionVariant::IfExp { .. } => "if exp".into(),
            TypedExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TypedExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.as_str())
            }
            TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" struct field access",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    field_to_access.name
                )
            }
            TypedExpressionVariant::IfLet {
                enum_type, variant, ..
            } => {
                format!(
                    "if let {}::{}",
                    enum_type.friendly_type_str(),
                    variant.name.as_str()
                )
            }
            TypedExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{}.{}\" tuple index",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    elem_to_access_num
                )
            }
            TypedExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TypedExpressionVariant::EnumInstantiation {
                tag,
                enum_decl,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_decl.name.as_str(),
                    variant_name.as_str(),
                    tag
                )
            }
            TypedExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TypedExpressionVariant::SizeOf { variant } => match variant {
                SizeOfVariant::Val(exp) => format!("size_of_val({:?})", exp.pretty_print()),
                SizeOfVariant::Type(type_name) => {
                    format!("size_of({:?})", type_name.friendly_type_str())
                }
            },
        }
    }
    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                function_body,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping));
                function_body.copy_types(type_mapping);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).copy_types(type_mapping);
                (*rhs).copy_types(type_mapping);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields.iter_mut().for_each(|x| x.copy_types(type_mapping)),
            Array { contents } => contents.iter_mut().for_each(|x| x.copy_types(type_mapping)),
            ArrayIndex { prefix, index } => {
                (*prefix).copy_types(type_mapping);
                (*index).copy_types(type_mapping);
            }
            StructExpression { fields, .. } => {
                fields.iter_mut().for_each(|x| x.copy_types(type_mapping))
            }
            CodeBlock(block) => {
                block.copy_types(type_mapping);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.copy_types(type_mapping);
                then.copy_types(type_mapping);
                if let Some(ref mut r#else) = r#else {
                    r#else.copy_types(type_mapping);
                }
            }
            AsmExpression {
                registers, //: Vec<TypedAsmRegisterDeclaration>,
                ..
            } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.copy_types(type_mapping));
            }
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                *resolved_type_of_parent = if let Some(matching_id) =
                    look_up_type_id(*resolved_type_of_parent).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*resolved_type_of_parent))
                };

                field_to_access.copy_types(type_mapping);
                prefix.copy_types(type_mapping);
            }
            IfLet {
                ref mut variant,
                ref mut enum_type,
                ..
            } => {
                *enum_type = if let Some(matching_id) =
                    look_up_type_id(*enum_type).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*enum_type))
                };
                variant.copy_types(type_mapping);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                *resolved_type_of_parent = if let Some(matching_id) =
                    look_up_type_id(*resolved_type_of_parent).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*resolved_type_of_parent))
                };

                prefix.copy_types(type_mapping);
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                enum_decl.copy_types(type_mapping);
                if let Some(ref mut contents) = contents {
                    contents.copy_types(type_mapping)
                };
            }
            AbiCast { address, .. } => address.copy_types(type_mapping),
            // storage is never generic and cannot be monomorphized
            StorageAccess { .. } => (),
            SizeOf { variant } => match variant {
                SizeOfVariant::Type(_) => (),
                SizeOfVariant::Val(exp) => exp.copy_types(type_mapping),
            },
        }
    }
}

use crate::semantic_analysis::{OwnedTypedStructField, TypeCheckedStorageAccess, TypedExpression};
use crate::{error::*, type_engine::TypeId, Ident};
use sway_types::{join_spans, Span};

#[derive(Clone, Debug)]
pub struct TypedStorageDeclaration {
    pub(crate) fields: Vec<TypedStorageField>,
    span: Span,
}

impl TypedStorageDeclaration {
    pub fn new(fields: Vec<TypedStorageField>, span: Span) -> Self {
        TypedStorageDeclaration { fields, span }
    }
    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_load(
        &self,
        field: Ident,
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        if let Some(TypedStorageField { r#type, name, .. }) = self
            .fields
            .iter()
            .find(|TypedStorageField { name, .. }| *name == field)
        {
            return ok(
                (TypeCheckedStorageAccess::new_load(name.clone()), *r#type),
                vec![],
                vec![],
            );
        } else {
            todo!("storage field not found err")
        }
    }
    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_store(
        &self,
        field: Ident,
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        if let Some(TypedStorageField { r#type, name, .. }) = self
            .fields
            .iter()
            .find(|TypedStorageField { name, .. }| *name == field)
        {
            return ok(
                (TypeCheckedStorageAccess::new_store(name.clone()), *r#type),
                vec![],
                vec![],
            );
        } else {
            todo!("storage field not found err")
        }
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub(crate) fn fields_as_owned_typed_struct_fields(&self) -> Vec<OwnedTypedStructField> {
        self.fields
            .iter()
            .map(
                |TypedStorageField {
                     ref name,
                     ref r#type,
                     ..
                 }| OwnedTypedStructField {
                    name: name.as_str().to_string(),
                    r#type: *r#type,
                },
            )
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct TypedStorageField {
    pub(crate) name: Ident,
    r#type: TypeId,
    initializer: TypedExpression,
}

impl TypedStorageField {
    pub fn new(name: Ident, r#type: TypeId, initializer: TypedExpression) -> Self {
        TypedStorageField {
            name,
            r#type,
            initializer,
        }
    }
}

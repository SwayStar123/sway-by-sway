use fuels_types::Property;
use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{namespace::Items, CopyTypes, TypeMapping},
    type_engine::{
        insert_type, look_up_type_id, look_up_type_id_raw, JsonAbiString, ToJsonAbi, TypeId,
    },
    CompileError, CompileResult, TypeInfo, TypeParameter, Visibility,
};

use super::{CreateTypeId, MonomorphizeHelper};

use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq)]
pub struct TypedStructDeclaration {
    pub(crate) name: Ident,
    pub(crate) fields: Vec<TypedStructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) visibility: Visibility,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.fields == other.fields
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TypedStructDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TypedStructDeclaration {
    fn type_id(&self) -> TypeId {
        insert_type(TypeInfo::Struct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl MonomorphizeHelper for TypedStructDeclaration {
    type Output = TypedStructDeclaration;

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn span(&self) -> &Span {
        &self.span
    }

    fn monomorphize_inner(self, type_mapping: &TypeMapping, namespace: &mut Items) -> Self::Output {
        let old_type_id = self.type_id();
        let mut new_decl = self;
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TypedStructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypedStructField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.r#type).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl CopyTypes for TypedStructField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.r#type = match look_up_type_id(self.r#type).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => insert_type(look_up_type_id_raw(self.r#type)),
        };
    }
}

impl TypedStructField {
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.to_string(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }

    pub(crate) fn expect_field_from_fields<'a>(
        struct_name: &'a Ident,
        fields: &'a [TypedStructField],
        field_to_access: &'a Ident,
    ) -> CompileResult<&'a TypedStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match fields
            .iter()
            .find(|TypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: fields
                        .iter()
                        .map(|TypedStructField { name, .. }| name.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    field_name: field_to_access.clone(),
                    struct_name: struct_name.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

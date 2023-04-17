use bluejay_core::definition::{
    AbstractBaseInputTypeReference, AbstractInputTypeReference, BaseInputTypeReference,
    EnumTypeDefinition, EnumValueDefinition, InputFieldsDefinition, InputObjectTypeDefinition,
    InputTypeReference, InputValueDefinition,
};
use bluejay_core::{AbstractValue, AsIter, BuiltinScalarDefinition, ObjectValue, Value};
use std::collections::BTreeMap;

mod error;

pub use error::Error;

#[derive(Debug, Clone, Copy)]
pub enum PathMember<'a> {
    Key(&'a str),
    Index(usize),
}

pub trait CoerceInput: AbstractInputTypeReference {
    fn coerce_value<'a, const CONST: bool, V: AbstractValue<CONST>>(
        &'a self,
        value: &'a V,
        path: &[PathMember<'a>],
    ) -> Result<(), Vec<Error<'a, CONST, V>>>;
}

impl<T: AbstractInputTypeReference> CoerceInput for T {
    fn coerce_value<'a, const CONST: bool, V: AbstractValue<CONST>>(
        &'a self,
        value: &'a V,
        path: &[PathMember<'a>],
    ) -> Result<(), Vec<Error<'a, CONST, V>>> {
        coerce_value_for_input_type_reference(self, value, path, true)
    }
}

fn coerce_value_for_input_type_reference<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    value: &'a V,
    path: &[PathMember<'a>],
    allow_implicit_list: bool,
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let core_type = input_type_reference.as_ref();
    let is_required = core_type.is_required();
    match value.as_ref() {
        Value::Null if is_required => Err(vec![Error::NullValueForRequiredType {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }]),
        Value::Null | Value::Variable(_) => Ok(()),
        core_value => match core_type {
            InputTypeReference::Base(_, _) => {
                coerce_value_for_base_input_type_reference(input_type_reference, value, path)
            }
            InputTypeReference::List(inner, _) => {
                if let Value::List(values) = core_value {
                    let errors: Vec<Error<'a, CONST, V>> = values
                        .iter()
                        .enumerate()
                        .flat_map(|(idx, value)| {
                            let mut path = path.to_owned();
                            path.push(PathMember::Index(idx));
                            coerce_value_for_input_type_reference(inner, value, &path, false)
                                .err()
                                .unwrap_or_default()
                        })
                        .collect();

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors)
                    }
                } else if allow_implicit_list {
                    coerce_value_for_input_type_reference(inner, value, path, true)
                } else {
                    Err(vec![Error::NoImplicitConversion {
                        value,
                        input_type_name: input_type_reference.as_ref().display_name(),
                        path: path.to_owned(),
                    }])
                }
            }
        },
    }
}

fn coerce_value_for_base_input_type_reference<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    let base = input_type_reference.as_ref().base().as_ref();
    match base {
        BaseInputTypeReference::BuiltinScalarType(bstd) => {
            coerce_builtin_scalar_value(input_type_reference, bstd, value, path)
        }
        BaseInputTypeReference::CustomScalarType(_) => Ok(()),
        BaseInputTypeReference::EnumType(etd) => {
            coerce_enum_value(input_type_reference, etd, value, path)
        }
        BaseInputTypeReference::InputObjectType(iotd) => {
            coerce_input_object_value(input_type_reference, iotd, value, path)
        }
    }
}

fn coerce_builtin_scalar_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    bstd: BuiltinScalarDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    match (bstd, value.as_ref()) {
        (BuiltinScalarDefinition::Boolean, Value::Boolean(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, Value::Float(_)) => Ok(()),
        (BuiltinScalarDefinition::Float, Value::Integer(_)) => Ok(()),
        (BuiltinScalarDefinition::ID, Value::Integer(_)) => Ok(()),
        (BuiltinScalarDefinition::ID | BuiltinScalarDefinition::String, Value::String(_)) => Ok(()),
        (BuiltinScalarDefinition::Int, Value::Integer(_)) => Ok(()),
        _ => Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }]),
    }
}

fn coerce_enum_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    enum_type_definition: &'a <T::BaseInputTypeReference as AbstractBaseInputTypeReference>::EnumTypeDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if let Value::Enum(name) = value.as_ref() {
        if enum_type_definition
            .enum_value_definitions()
            .iter()
            .any(|evd| evd.name() == name)
        {
            Ok(())
        } else {
            Err(vec![Error::NoEnumMemberWithName {
                name,
                value,
                enum_type_name: enum_type_definition.name(),
                path: path.to_owned(),
            }])
        }
    } else {
        Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }])
    }
}

fn coerce_input_object_value<
    'a,
    const CONST: bool,
    V: AbstractValue<CONST>,
    T: AbstractInputTypeReference,
>(
    input_type_reference: &'a T,
    input_object_type_definition: &'a <T::BaseInputTypeReference as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    value: &'a V,
    path: &[PathMember<'a>],
) -> Result<(), Vec<Error<'a, CONST, V>>> {
    if let Value::Object(object) = value.as_ref() {
        let mut errors = Vec::new();
        let mut missing_required_values = Vec::new();

        type Entry<'a, const CONST: bool, V> = (
            &'a <<V as AbstractValue<CONST>>::Object as ObjectValue<CONST>>::Key,
            &'a V,
        );
        let indexed_object: BTreeMap<&'a str, Vec<Entry<'a, CONST, V>>> =
            object
                .iter()
                .fold(BTreeMap::new(), |mut index, (key, value)| {
                    index.entry(key.as_ref()).or_default().push((key, value));
                    index
                });

        errors.extend(indexed_object.iter().filter_map(|(&field_name, entries)| {
            (entries.len() > 1).then(|| Error::NonUniqueFieldNames {
                value,
                field_name,
                keys: Vec::from_iter(entries.iter().map(|&(key, _)| key)),
                path: path.to_owned(),
            })
        }));

        errors.extend(object.iter().filter_map(|(field, _)| {
            input_object_type_definition
                .input_field_definitions()
                .get(field.as_ref())
                .is_none()
                .then(|| Error::NoInputFieldWithName {
                    field,
                    input_object_type_name: input_object_type_definition.name(),
                    path: path.to_owned(),
                })
        }));

        input_object_type_definition
            .input_field_definitions()
            .iter()
            .for_each(|ivd| {
                let value_for_field = indexed_object
                    .get(ivd.name())
                    .and_then(|entries| entries.first().copied().map(|(_, value)| value));
                let default_value = ivd.default_value();

                match (value_for_field, default_value) {
                    (None, None) => {
                        if ivd.r#type().as_ref().is_required() {
                            missing_required_values.push(ivd.name());
                        }
                    }
                    (None, Some(_)) => {}
                    (Some(value), _) => {
                        let mut inner_path = path.to_owned();
                        inner_path.push(PathMember::Key(ivd.name()));
                        match ivd.r#type().coerce_value(value, &inner_path) {
                            Ok(_) => {}
                            Err(errs) => errors.extend(errs),
                        }
                    }
                }
            });

        if !missing_required_values.is_empty() {
            errors.push(Error::NoValueForRequiredFields {
                value,
                field_names: missing_required_values,
                input_object_type_name: input_object_type_definition.name(),
                path: path.to_owned(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    } else {
        Err(vec![Error::NoImplicitConversion {
            value,
            input_type_name: input_type_reference.as_ref().display_name(),
            path: path.to_owned(),
        }])
    }
}

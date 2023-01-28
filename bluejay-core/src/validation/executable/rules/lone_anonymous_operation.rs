use std::marker::PhantomData;
use crate::validation::executable::{Visitor, Error, Rule};
use crate::executable::{
    ExecutableDocument,
    OperationDefinitionFromExecutableDocument,
};
use crate::definition::SchemaDefinition;

pub struct LoneAnonymousOperation<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
    anonymous_operations: Vec<&'a OperationDefinitionFromExecutableDocument<'a, E>>,
    executable_document: &'a E,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Visitor<'a, E, S> for LoneAnonymousOperation<'a, E, S> {
    fn visit_operation(&mut self, operation_definition: &'a OperationDefinitionFromExecutableDocument<'a, E>) {
        if operation_definition.name().is_none() {
            self.anonymous_operations.push(operation_definition);
        }
    }
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> IntoIterator for LoneAnonymousOperation<'a, E, S> {
    type Item = Error<'a, E, S>;
    type IntoIter = <Option<Error<'a, E, S>> as IntoIterator>::IntoIter;
 
    fn into_iter(self) -> Self::IntoIter {
        (self.executable_document.operation_definitions().len() != 1 && !self.anonymous_operations.is_empty())
            .then(|| Error::NotLoneAnonymousOperation { anonymous_operations: self.anonymous_operations })
            .into_iter()
    }
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Rule<'a, E, S> for LoneAnonymousOperation<'a, E, S> {
    fn new(executable_document: &'a E, _: &'a S) -> Self {
        Self { anonymous_operations: Vec::new(), executable_document, schema_definition: Default::default() }
    }
}
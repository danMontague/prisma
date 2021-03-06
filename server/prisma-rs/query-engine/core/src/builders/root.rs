use super::Builder;
use crate::{CoreResult, ReadQuery};
use graphql_parser::query::*;
use prisma_models::SchemaRef;
use std::sync::Arc;

#[derive(Debug)]
pub struct RootBuilder {
    pub query: Document,
    pub schema: SchemaRef,
    pub operation_name: Option<String>,
}

impl RootBuilder {
    // FIXME: Find op name and only execute op!
    pub fn build(self) -> CoreResult<Vec<ReadQuery>> {
        self.query
            .definitions
            .iter()
            .map(|d| match d {
                // Query without the explicit "query" before the selection set
                Definition::Operation(OperationDefinition::SelectionSet(SelectionSet { span: _, items })) => {
                    self.build_query(&items)
                }

                // Regular query
                Definition::Operation(OperationDefinition::Query(Query {
                    position: _,
                    name: _,
                    variable_definitions: _,
                    directives: _,
                    selection_set,
                })) => self.build_query(&selection_set.items),
                _ => unimplemented!(),
            })
            .collect::<CoreResult<Vec<Vec<ReadQuery>>>>() // Collect all the "query trees"
            .map(|v| v.into_iter().flatten().collect())
    }

    fn build_query(&self, root_fields: &Vec<Selection>) -> CoreResult<Vec<ReadQuery>> {
        root_fields
            .iter()
            .map(|item| {
                // First query-level fields map to a model in our schema, either a plural or singular
                match item {
                    Selection::Field(root_field) => Builder::new(Arc::clone(&self.schema), root_field)?.build(),
                    _ => unimplemented!(),
                }
            })
            .collect()
    }
}

trait UuidCheck {
    fn is_uuid(&self) -> bool;
}

impl UuidCheck for String {
    fn is_uuid(&self) -> bool {
        false
    }
}

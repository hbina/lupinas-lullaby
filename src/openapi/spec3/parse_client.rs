use std::collections::HashMap;

use crate::openapi::{from_path, use_spec};

use super::{
    spec::{ObjectOrReference, Operation, Parameter, ParameterLocation, Path},
    Spec3,
};

#[derive(Debug)]
enum Intermediate<T> {
    Alias(String),
    Inline(T),
}

#[derive(Debug)]
struct QueryParameter {
    required: bool,
    deprecated: bool,
}

impl QueryParameter {
    pub fn from_parameter(parameter: &Parameter) -> QueryParameter {
        QueryParameter {
            required: parameter.required.unwrap_or(false),
            deprecated: parameter.deprecated.unwrap_or(false),
        }
    }
}

#[derive(Debug)]
enum ParameterTypes {
    Query(QueryParameter),
    Header,
    Path,
    Cookie,
}

#[derive(Debug)]
struct HttpOperation {
    queries: HashMap<String, QueryParameter>,
}

#[derive(Debug)]
struct HttpOperations {
    get: Option<HttpOperation>,
    put: Option<HttpOperation>,
    post: Option<HttpOperation>,
    delete: Option<HttpOperation>,
}

fn parse_operation(path: &Operation) -> ! {
    let parameters = if let Some(parameters) = path.parameters.as_ref() {
        parameters
            .iter()
            .map(|p| match p {
                ObjectOrReference::Ref(r) => Intermediate::Alias(r.ref_path.clone()),
                ObjectOrReference::Object(o) => match o.location {
                    ParameterLocation::Query => Intermediate::Inline(ParameterTypes::Query(
                        QueryParameter::from_parameter(o),
                    )),
                    ParameterLocation::Header => Intermediate::Inline(ParameterTypes::Header),
                    ParameterLocation::Path => Intermediate::Inline(ParameterTypes::Path),
                    ParameterLocation::Cookie => Intermediate::Inline(ParameterTypes::Cookie),
                },
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    let response = path.responses.iter().map(|(status_code, response)| {
        let result = match response {
            ObjectOrReference::Ref(r) => Intermediate::Alias((status_code, r.ref_path.clone())),
            ObjectOrReference::Object(o) => Intermediate::Inline(),
        };
    });
    unimplemented!()
}

fn parse_path(path: &Path) -> Intermediate<HttpOperations> {
    if let Some(reference) = path.ref_path.as_ref() {
        Intermediate::Alias(String::from(reference))
    } else {
        let operations = path.get.as_ref().map(parse_operation);
        println!("operations:\n{:#?}", operations);
        unimplemented!()
    }
}

pub fn generate_clients(spec: &Spec3) -> String {
    let paths = spec
        .paths
        .iter()
        .map(|(name, path)| parse_path(path))
        .collect::<Vec<_>>();
    unimplemented!()
}

#[test]
pub fn test_v2_json_client_examples() {
    let path = std::path::Path::new("./data/v3.0/petstore.json");
    match from_path(path) {
        crate::openapi::OpenApi::V2(_) => {
            panic!("Wrong file")
        }
        crate::openapi::OpenApi::V3(spec) => println!("{}", generate_clients(&spec)),
    }
}

use super::{
    parse_type::{
        self, parse_option_schema_objectref_to_javascript_type, parse_reference,
        parse_schema_object_to_javascript_type,
    },
    spec::{
        HeaderObj, MediaTypeObj, ObjectOrReference, OperationObj, ParameterLocation, ParameterObj,
        PathObj, ResponseObj, SchemaObj,
    },
    types::{JavaScriptType, RowTriplet},
    Spec3,
};
use crate::openapi::{from_path, use_spec};
use parse_type::{
    parse_header_object_to_row_triplet, parse_media_type_object_to_javascript_type,
    parse_option_schema_object_to_javascript_type, parse_response_objectref_to_javascript_type,
    parse_schema_object_to_javascript_arrays, parse_schema_object_to_javascript_row_triplets,
};
use serde_json::{value, Value};
use std::collections::{BTreeMap, HashMap};

const REFERENCE_UNSUPPORTED_ERROR_STRING:&'static str = "The ability to resolve paths based on the name is still not implemented. If you have a sample Swagger 3.0 file that uses this feature then raise an issue!";

fn unwrap_object_reference_f<Fin, T, O>(f: Fin) -> impl FnMut(&ObjectOrReference<T>) -> O
where
    Fin: Fn(&T) -> O,
{
    move |ref obj| match obj {
        ObjectOrReference::Ref(_) => {
            unimplemented!("The ability to resolve references is not yet implemented")
        }
        ObjectOrReference::Object(ref o) => f(o),
    }
}

#[derive(Debug)]
pub struct Argument {
    pub queries: Vec<RowTriplet>,
    pub paths: Vec<RowTriplet>,
    pub headers: Vec<RowTriplet>,
    pub cookies: Vec<RowTriplet>,
}

impl Argument {
    pub fn new() -> Argument {
        Argument {
            queries: vec![],
            paths: vec![],
            headers: vec![],
            cookies: vec![],
        }
    }
}

fn parse_operation_arguments(operation: &OperationObj) -> Argument {
    let mut argument = Argument::new();
    println!("operation:{:#?}", operation);
    for parameter in operation
        .parameters
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|parameter| match parameter {
            ObjectOrReference::Object(o) => Some(o),
            _ => None,
        })
    {
        let triplet = RowTriplet::from_triplet(
            parameter.name.clone(),
            parameter.required.unwrap_or(false),
            parameter
                .schema
                .as_ref()
                .map(unwrap_object_reference_f(|f| {
                    parse_type::parse_schema_object_to_javascript_type(f)
                }))
                .unwrap_or(JavaScriptType::typename("any")),
        );
        match parameter.location {
            ParameterLocation::Query => argument.queries.push(triplet),
            ParameterLocation::Header => argument.headers.push(triplet),
            ParameterLocation::Path => argument.paths.push(triplet),
            ParameterLocation::Cookie => argument.cookies.push(triplet),
        }
    }
    argument
}

#[derive(Debug)]
pub struct Response {
    status_code: String,
    ttype: JavaScriptType,
}

fn parse_operation_responses(operation: &OperationObj) -> Vec<Response> {
    operation
        .responses
        .iter()
        .map(|(status_code, obj)| Response {
            status_code: status_code.clone(),
            ttype: parse_response_objectref_to_javascript_type(obj),
        })
        .filter(|s| s.status_code != "default")
        .collect::<Vec<_>>()
}

fn replace_path_with_argument(argument: &Argument, string: &String) -> String {
    println!("argument:\n{:#?}\nstring:\n{:#?}", argument, string);
    if let Some(r) = argument
        .queries
        .iter()
        .map(|x| &x.name)
        .find(|x| x == &string)
    {
        format!("queries.{}", r)
    } else if let Some(r) = argument
        .paths
        .iter()
        .map(|x| &x.name)
        .find(|x| x == &string)
    {
        format!("paths.{}", r)
    } else if let Some(r) = argument
        .headers
        .iter()
        .map(|x| &x.name)
        .find(|x| x == &string)
    {
        format!("headers.{}", r)
    } else if let Some(r) = argument
        .cookies
        .iter()
        .map(|x| &x.name)
        .find(|x| x == &string)
    {
        format!("cookies.{}", r)
    } else {
        panic!(format!(
            "Cannot find that parameter name '{}' anywhere",
            string
        ))
    }
}

fn parse_operation_link(path: &String, argument: &Argument) -> String {
    #[derive(Debug)]
    pub enum Item {
        String(String),
        Reference(String),
    }
    let items = path
        .split('/')
        .map(|x| {
            if x.starts_with("{") && x.ends_with("}") {
                Item::Reference(String::from(&x[1..x.len() - 1]))
            } else {
                Item::String(String::from(x))
            }
        })
        .map(|i| match i {
            Item::String(s) => s,
            Item::Reference(r) => format!("${{{}}}", replace_path_with_argument(argument, &r)),
        })
        .collect::<Vec<String>>()
        .join("/");
    items
}

pub fn create_client_function(
    instance_name: &String,
    name: &String,
    arguments: &Argument,
    responses: &Vec<Response>,
    path_link: &String,
) -> String {
    let config_queries_signature = format!(
        r##"{{{}}}"##,
        arguments
            .queries
            .iter()
            .map(|s| { s.to_string() })
            .collect::<Vec<String>>()
            .join(",")
    );
    let config_paths_signature = format!(
        r##"{{{}}}"##,
        arguments
            .paths
            .iter()
            .map(|s| { s.to_string() })
            .collect::<Vec<String>>()
            .join(",")
    );
    let config_head_signature = format!("{{}}");
    let config_cookies_signature = format!("{{}}");
    let return_sum_types = format!(
        r##"{}"##,
        responses
            .iter()
            .map(|s| {
                format!(
                    r##"
{{
    status : {},
    body : {}
}}"##,
                    s.status_code, s.ttype
                )
            })
            .collect::<Vec<String>>()
            .join("|")
    );
    let switch_body = format!(
        r##"{}"##,
        responses
            .iter()
            .map(|s| {
                format!(
                    r##"
                case {}: {{
                    return {{
                      status: {},
                      body: result.data,
                    }};
                  }}
            "##,
                    s.status_code, s.status_code
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    );
    format!(
        r##"export async function {}(
    params: {},
    paths: {},
    headers: {},
    cookies: {}
): Promise<{}> {{
    const result = await {}.get(`{}`,
    {{
        params,
        headers,
    }});
    switch (result.status) {{
{}
    }}
    throw new Error();
}}"##,
        name,
        config_queries_signature,
        config_paths_signature,
        config_head_signature,
        config_cookies_signature,
        return_sum_types,
        instance_name,
        path_link,
        switch_body
    )
}

pub fn generate_clients(spec: &Spec3) -> String {
    let instances = spec
        .servers
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            format!(
                r##"
const server{} = axios.create({{
baseURL: "{}",
}});"##,
                idx, s.url
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let clients = (0..spec.servers.iter().enumerate().len())
        .map(|idx| {
            let instance_name = format!("server{}", idx);
            spec.paths
                .iter()
                .map(|(path, path_obj)| {
                    if let Some(_) = path_obj.ref_path.as_ref() {
                        unimplemented!("{}", REFERENCE_UNSUPPORTED_ERROR_STRING);
                    } else {
                        let get = path_obj
                            .get
                            .as_ref()
                            .map(|operation| {
                                let name = operation.operation_id.as_ref().unwrap();
                                let arguments = parse_operation_arguments(operation);
                                let responses = parse_operation_responses(operation);
                                let link = parse_operation_link(path, &arguments);
                                create_client_function(
                                    &instance_name,
                                    name,
                                    &arguments,
                                    &responses,
                                    &link,
                                )
                            })
                            .unwrap_or(String::new());
                        get
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("{}{}", instances, clients)
}

#[test]
pub fn test_v3_json_client_examples() {
    let path = std::path::Path::new("./data/v3.0/petstore.json");
    match from_path(path) {
        crate::openapi::OpenApi::V2(_) => {
            panic!("Wrong file")
        }
        crate::openapi::OpenApi::V3(spec) => println!("{}", generate_clients(&spec)),
    }
}

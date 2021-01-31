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
    parse_option_schema_object_to_javascript_type, parse_response_object_to_javascript_type,
    parse_schema_object_to_javascript_arrays, parse_schema_object_to_javascript_row_triplets,
};
use std::{
    any::type_name,
    collections::{BTreeMap, HashMap},
};

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

fn parse_operation(operation: &OperationObj) -> String {
    let name = operation.operation_id.as_ref().expect("Please consider giving this operation a name. We have not figured out a way to nicely produce a name for an operation.");
    let queries_param = operation
        .parameters
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(unwrap_object_reference_f(|p: &ParameterObj| {
            match p.location {
                ParameterLocation::Query => Some((
                    p.name.clone(),
                    p.required.unwrap_or(false),
                    p.schema
                        .as_ref()
                        .map(unwrap_object_reference_f(|f| {
                            parse_type::parse_schema_object_to_javascript_type(f)
                        }))
                        .unwrap_or(JavaScriptType::typename("any")),
                )),
                _ => None,
            }
        }))
        .collect::<Vec<_>>();
    let responses = operation
        .responses
        .iter()
        .map(|(status_code, obj)| {
            (
                status_code.clone(),
                parse_response_object_to_javascript_type(obj),
            )
        })
        .collect::<Vec<_>>();
    println!("responses:{:#?}", responses);
    let result = format!(
        r##"
    export async function
    {}
    (
        queries:{{
            {}
        }}
    ) : Promise<
    {}
    > // Figure out the sum type here
    {{
        try {{
            const result = await instance.get("/", {{
                params : queries
            }});
            switch (result.status) {{
                case 200: {{
                  return {{
                    status: 200,
                    body: result.data,
                  }};
                }}
              }}
        }} catch (e) {{
            throw e;
        }}
    }} "##,
        name,
        queries_param
            .iter()
            .map(|x| format!("{} {} : {}", x.0, if x.1 { "" } else { "?" }, x.2))
            .collect::<Vec<_>>()
            .join(","),
        responses
            .iter()
            .map(|(status_code, ttype)| {
                format!(
                    r##"
                {{
                    status: {},
                    data : {}
                }}
                "##,
                    status_code, ttype
                )
            })
            .collect::<Vec<String>>()
            .join("|")
    );
    println!("result\n:{}", result);
    unimplemented!()
}

pub fn generate_clients(spec: &Spec3) -> String {
    spec.paths
        .iter()
        .map(|(path, path_obj)| {
            if let Some(reference) = path_obj.ref_path.as_ref() {
        unimplemented!("The ability to resolve paths based on the name is still not implemented. If you have a sample Swagger 3.0 file that uses this feature then raise an issue!");
            } else {
                let get = path_obj.get.as_ref().map(parse_operation);
            }
        })
        .for_each(|x|{});
    spec.servers.iter().enumerate().for_each(|(idx, s)| {
        println!(
            r##"
const server{} = axios.create({{
    baseURL: "{}",
}});"##,
            idx, s.url
        )
    });
    format!(r##"types"##,)
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

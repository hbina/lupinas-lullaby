use super::{
    spec::{
        HeaderObj, MediaTypeObj, ObjectOrReference, OperationObj, ParameterLocation, ParameterObj,
        PathObj, ResponseObj, SchemaObj,
    },
    Spec3,
};
use crate::openapi::{from_path, use_spec};
use std::collections::{BTreeMap, HashMap};

fn parse_reference(reference: &String) -> String {
    let (prefix, name) = reference.split_at("#/components/schemas/".len());
    if prefix == "#/components/schemas/" {
        String::from(name)
    } else {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Intermediate<T> {
    Alias(String),
    Inline(T),
}

#[derive(Debug)]
pub struct ParamterContent {
    name: String,
    required: bool,
    deprecated: bool,
}

impl ParamterContent {
    pub fn from_parameter_object(parameter: &ParameterObj) -> ParamterContent {
        ParamterContent {
            name: parameter.name.clone(),
            required: parameter.required.unwrap_or(false),
            deprecated: parameter.deprecated.unwrap_or(false),
        }
    }
}

#[derive(Debug)]
pub enum ParameterType {
    Query(ParamterContent),
    Header(ParamterContent),
    Path(ParamterContent),
    Cookie(ParamterContent),
}

impl ParameterType {
    pub fn from_parameter_types_objectref(o: &ObjectOrReference<ParameterObj>) -> ParameterType {
        match o {
            ObjectOrReference::Ref(r) => unimplemented!(
                r##"
The intermediate step currently does not currently have support for an alias to a parameter object.
If you happen to have a sample Swagger 3.0 file that have uses this feature please let me know :)"##
            ),
            ObjectOrReference::Object(o) => match o.location {
                ParameterLocation::Query => {
                    ParameterType::Query(ParamterContent::from_parameter_object(o))
                }
                ParameterLocation::Header => {
                    ParameterType::Header(ParamterContent::from_parameter_object(o))
                }
                ParameterLocation::Path => {
                    ParameterType::Path(ParamterContent::from_parameter_object(o))
                }
                ParameterLocation::Cookie => {
                    ParameterType::Cookie(ParamterContent::from_parameter_object(o))
                }
            },
        }
    }
}

#[derive(Debug)]
struct HttpOperations {
    path: String,
    get: Option<Operation>,
    put: Option<Operation>,
    post: Option<Operation>,
    delete: Option<Operation>,
}

#[derive(Debug)]
pub struct Header {
    required: bool,
    schema: Option<ObjectOrReference<SchemaObj>>,
}

impl Header {
    pub fn from_header_objectref(o: &ObjectOrReference<HeaderObj>) -> Intermediate<Header> {
        match o {
            ObjectOrReference::Ref(r) => Intermediate::Alias(r.ref_path.clone()),
            ObjectOrReference::Object(o) => Intermediate::Inline(Header {
                required: o.required.unwrap_or(false),
                schema: o.schema.clone(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Media {
    schema: SchemaObj,
}

impl Media {
    pub fn from_media_type_object(o: &MediaTypeObj) -> Option<Intermediate<Media>> {
        o.schema.as_ref().map(|s| match s {
            ObjectOrReference::Ref(r) => Intermediate::Alias(r.ref_path.clone()),
            ObjectOrReference::Object(o) => Intermediate::Inline(Media { schema: o.clone() }),
        })
    }
}

#[derive(Debug)]
pub struct Response {
    description: String,
    headers: BTreeMap<String, Intermediate<Header>>,
    contents: BTreeMap<String, Intermediate<Media>>,
}

impl Response {
    pub fn from_response_object(obj: &ResponseObj) -> Response {
        let headers = if let Some(headers) = obj.headers.as_ref() {
            headers
                .iter()
                .map(|(name, objref)| (name.clone(), Header::from_header_objectref(objref)))
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        let contents = if let Some(contents) = obj.content.as_ref() {
            contents
                .iter()
                .filter_map(|(name, media)| {
                    Media::from_media_type_object(media).map(|media| (name.clone(), media))
                })
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        let description = obj.description.clone();
        Response {
            description,
            headers,
            contents,
        }
    }

    pub fn from_response_objectref(obj: &ObjectOrReference<ResponseObj>) -> Intermediate<Response> {
        match obj {
            ObjectOrReference::Ref(r) => Intermediate::Alias(r.ref_path.clone()),
            ObjectOrReference::Object(o) => Intermediate::Inline(Response::from_response_object(o)),
        }
    }
}

#[derive(Debug)]
pub struct Operation {
    operation_id: String,
    parameters: Vec<ParameterType>,
    responses: BTreeMap<String, Intermediate<Response>>,
}

impl Operation {
    pub fn parse_operation(path: &OperationObj) -> Operation {
        let operation_id = path
            .operation_id
            .as_ref()
            .expect(
                format!(
                    "Unnammed method is currently not supported. object:\n{:#?}",
                    path,
                )
                .as_str(),
            )
            .clone();
        let parameters = if let Some(parameters) = path.parameters.as_ref() {
            parameters
                .iter()
                .map(ParameterType::from_parameter_types_objectref)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let responses = path
            .responses
            .iter()
            .map(|(status_code, response)| {
                (
                    status_code.clone(),
                    Response::from_response_objectref(response),
                )
            })
            .collect::<BTreeMap<_, _>>();
        Operation {
            operation_id,
            parameters,
            responses,
        }
    }

    pub fn into_type_string_template(&self) -> String {
        self.responses
            .iter()
            .map(|(status_code, r)| {
                let tmp = match r {
                    Intermediate::Alias(a) => parse_reference(a),
                    Intermediate::Inline(r) => r
                        .contents
                        .iter()
                        .map(|(media_type, media)| match media_type.as_str() {
                            "application/json" => match media {
                                Intermediate::Alias(a) => parse_reference(a),
                                Intermediate::Inline(o) => {
                                    format!("schema:{:?}", o.schema)
                                }
                            },
                            _ => unimplemented!(),
                        })
                        .collect::<String>(),
                };
                format!(
                    r##"
async function {}(): {{
    status:{},
    body : {}
}} {{
    // function definition
}}
            "##,
                    self.operation_id, status_code, tmp
                )
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}

fn parse_path(path: &String, path_obj: &PathObj) -> HttpOperations {
    let get = path_obj.get.as_ref().map(Operation::parse_operation);
    let put = path_obj.put.as_ref().map(Operation::parse_operation);
    let post = path_obj.post.as_ref().map(Operation::parse_operation);
    let delete = path_obj.delete.as_ref().map(Operation::parse_operation);
    HttpOperations {
        path: path.clone(),
        get,
        put,
        post,
        delete,
    }
}

pub struct Path {
    name: String,
    operation: Intermediate<HttpOperations>,
}

pub fn generate_clients(spec: &Spec3) -> String {
    spec.paths
        .iter()
        .map(|(path, path_obj)| parse_path(path, path_obj))
        .map(|p| {
            if let Some(get) = p.get {
                get.into_type_string_template()
            } else {
                format!("No GET method")
            }
        })
        .for_each(|x| println!("{}", x));
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

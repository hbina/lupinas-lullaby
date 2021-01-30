use crate::openapi::spec3::{
    spec::{ObjectOrReference, Ref, SchemaObj},
    Spec3,
};

use super::types::{JavaScriptConstruct, JavaScriptType, RowTriplet, Spec3Error};

fn parse_reference(reference: &Ref) -> String {
    let (prefix, name) = reference.ref_path.split_at("#/components/schemas/".len());
    if prefix == "#/components/schemas/" {
        String::from(name)
    } else {
        panic!("{:#?}", Spec3Error::InvalidReference(reference.clone()))
    }
}

pub fn parse_schema_object_to_javascript_arrays(schema: &SchemaObj) -> JavaScriptType {
    if let Some(x) = schema.items.as_ref() {
        match x.as_ref() {
            ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
            ObjectOrReference::Ref(s) => JavaScriptType::Typename(parse_reference(s)),
        }
    } else {
        panic!(
            "{:#?}",
            Spec3Error::CannotConvertSchemaToArray(schema.clone())
        )
    }
}

pub fn parse_schema_object_to_javascript_strings(schema: &SchemaObj) -> JavaScriptType {
    if let Some(enums) = schema.enum_values.as_ref() {
        JavaScriptType::Sum(enums.clone())
    } else {
        // TODO(hbina): Handle the case for enums
        if let Some(format) = schema.format.as_ref() {
            match format.as_str() {
                "date" | "date-time" => JavaScriptType::typename("Date"),
                _ => JavaScriptType::typename("String"),
            }
        } else {
            JavaScriptType::typename("String")
        }
    }
}

pub fn parse_schema_object_to_javascript_row_triplets(schema: &SchemaObj) -> Vec<RowTriplet> {
    // 1. Find the required properties.
    // 2. Iterate through properties.
    // 3. Parse each rows type, creating a triplet of (name, required, type)
    if let Some(properties) = schema.properties.as_ref() {
        let required = schema.required.as_ref();
        let result = properties
            .iter()
            .map(|(name, object)| {
                let name = name.to_string();
                let row_required = required.map(|r| r.contains(&name)).unwrap_or(false);
                let ttype = match object {
                    ObjectOrReference::Ref(r) => JavaScriptType::Typename(parse_reference(r)),
                    ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
                };
                RowTriplet::from_triplet(name, row_required, ttype)
            })
            .collect::<Vec<RowTriplet>>();
        result
    } else {
        vec![]
    }
}

// TODO(hbina): Reimplement this to return an intermediate object so we can log the transformation.
pub fn parse_schema_object_to_javascript_type(schema: &SchemaObj) -> JavaScriptType {
    if let Some(ttype) = schema.schema_type.as_ref() {
        match ttype.as_str() {
            "array" => {
                JavaScriptType::Array(Box::new(parse_schema_object_to_javascript_arrays(schema)))
            }
            "string" => parse_schema_object_to_javascript_strings(schema),
            "object" => JavaScriptType::AnonymousObject(
                parse_schema_object_to_javascript_row_triplets(schema),
            ),
            // TODO(hbina): Narrow down the exact type later.
            "integer" | "number" => JavaScriptType::typename("number"),
            "boolean" => JavaScriptType::typename("boolean"),
            "unknown" => JavaScriptType::typename("unknown"),
            // TODO(hbina): It is entirely possile type of a schema object to just be a string.
            // I should think.
            // Actually, this case should not even be possible because `types` can take a limited set of values.
            // Rework `openapi` to make this unrepresentable.
            _ => panic!(
                r##"attempting to parse schema with unknown type. schema:{:#?}"##,
                schema
            ),
        }
    } else {
        // TODO(hbina): Revisit this case.
        // The specification does not say anything about the absent of this value.
        // It might be inherited from JSON SchemaObject. Look it up.
        JavaScriptType::typename("any")
    }
}

pub fn parse_option_schema_object_to_javascript_type(schema: Option<&SchemaObj>) -> JavaScriptType {
    if let Some(schema) = schema {
        parse_schema_object_to_javascript_type(schema)
    } else {
        JavaScriptType::typename("any")
    }
}

pub fn parse_root_schema_object_to_javascript_construct(
    (name, schema): (&String, &ObjectOrReference<SchemaObj>),
) -> JavaScriptConstruct {
    let name = name.to_string();
    match schema {
        ObjectOrReference::Ref(s) => {
            JavaScriptConstruct::Alias(name, JavaScriptType::Typename(parse_reference(s)))
        }
        ObjectOrReference::Object(v) => {
            let body = parse_schema_object_to_javascript_type(v);
            JavaScriptConstruct::Alias(name, body)
        }
    }
}

pub fn generate_types(spec: &Spec3) -> String {
    if let Some(components) = spec.components.as_ref() {
        if let Some(schemas) = components.schemas.as_ref() {
            let result = schemas
                .iter()
                .map(parse_root_schema_object_to_javascript_construct)
                .collect::<Vec<_>>()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<String>();
            return result;
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

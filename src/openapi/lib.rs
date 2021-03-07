mod repr;
mod spec2;
mod spec3;

use self::repr::filter_empty_types;
use self::{
    spec2::{use_spec2, Spec2},
    spec3::{use_spec3, Spec3},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum OpenApi {
    V2(Spec2),
    V3(Spec3),
}

pub fn from_path<P>(path: P) -> OpenApi
where
    P: AsRef<Path>,
{
    from_reader(File::open(path).unwrap())
}

pub fn from_reader<R>(read: R) -> OpenApi
where
    R: Read,
{
    serde_yaml::from_reader::<R, OpenApi>(read).unwrap()
}

pub fn from_bytes(read: &[u8]) -> OpenApi {
    serde_yaml::from_slice::<OpenApi>(read).unwrap()
}

pub fn use_spec(spec: &OpenApi, skip: bool) -> String {
    let types = match spec {
        OpenApi::V2(spec) => use_spec2(spec),
        OpenApi::V3(spec) => use_spec3(spec),
    };
    types
        .into_iter()
        .filter_map(|(name, tt)| {
            if skip {
                let result = filter_empty_types(tt).map(|tt| (name, tt));
                result
            } else {
                Some((name, tt))
            }
        })
        .map(|(name, ttype)| format!("export type {} = {};", name, ttype))
        .collect::<Vec<String>>()
        .join("\n")
}

#[test]
pub fn test_v2_json_examples() {
    let _result = std::fs::read_dir("./data/v2.0/json")
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.is_file())
        .map(|x| from_path(x))
        .map(|spec| use_spec(&spec, false))
        .collect::<Vec<_>>();
}

#[test]
pub fn test_v2_yaml_examples() {
    let _result = std::fs::read_dir("./data/v2.0/yaml")
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.is_file())
        .map(|x| from_path(x))
        .map(|spec| use_spec(&spec, false))
        .collect::<Vec<_>>();
}

#[test]
pub fn test_v3_examples() {
    let _result = std::fs::read_dir("./data/v3.0")
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.is_file())
        .map(|x| from_path(x))
        .map(|spec| use_spec(&spec, false))
        .collect::<Vec<_>>();
}

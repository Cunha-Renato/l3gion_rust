use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::MyError;

pub trait NodeInfo {}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct YamlNode {
    pub name: String,
    pub node_type: String,
    pub value: String,
    pub children: Vec<YamlNode>,
}
impl YamlNode {
    pub fn push(&mut self, other: Self) {
        self.children.push(other);
    }
    pub fn serialize(&self, path: &str, name: &str) -> Result<(), MyError> {
        serialize(&self, path, name)?;
        Ok(())
    }
    pub fn deserialize(path: &str, name: &str) -> Result<Self, MyError> {
        deserialize(path, name)
    }
}

pub fn serialize<T: Serialize>(value: &T, path: &str, name: &str) -> Result<(), MyError> {
    let path = path.to_string() + name;
    std::fs::write(path, serde_yaml::to_string(value)?)?;
    Ok(())
}
pub fn deserialize<T: DeserializeOwned>(path: &str, name: &str) -> Result<T, MyError> {
    let path = path.to_string() + name + ".yaml";
    let content = std::fs::read_to_string(path)?;
    let result: T = serde_yaml::from_str(&content)?;

    Ok(result)
}
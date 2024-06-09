type StdError = Box<dyn std::error::Error>;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
    pub fn serialize(&self, path: &str, name: &str) -> Result<(), StdError> {
        serialize(&self, path, name)?;
        Ok(())
    }
    pub fn deserialize(path: &str, name: &str) -> Result<Self, StdError> {
        deserialize(path, name)
    }
    pub fn deserialize_full_path(path: &str) -> Result<Self, StdError> {
        deserialize_full_path(path)
    }
}

pub fn serialize<T: Serialize>(value: &T, path: &str, name: &str) -> Result<(), StdError> {
    let path = format!("{}{}.yaml", path, name);
    std::fs::write(path, serde_yaml::to_string(value)?)?;
    Ok(())
}
pub fn deserialize<T: DeserializeOwned>(path: &str, name: &str) -> Result<T, StdError> {
    let path = format!("{}{}.yaml", path, name);
    let content = std::fs::read_to_string(path)?;
    let result: T = serde_yaml::from_str(&content)?;

    Ok(result)
}
pub fn deserialize_full_path<T: DeserializeOwned>(path: &str) -> Result<T, StdError> {
    // let content = std::path::Path::new(path);
    let file = std::fs::File::open(path)?;
    Ok(serde_yaml::from_reader::<std::fs::File, T>(file)?)
    // Ok(serde_yaml::from_str(&content)?)
}
use std::collections::HashMap;
use crate::level::Level;
pub fn add_default_levels(levels: &mut HashMap<String, Level>) -> bool {
levels.insert("end".to_string(), toml::from_str::<Level>(&include_str!("../resources/levels/end.toml")).unwrap());
levels.insert("level_1".to_string(), toml::from_str::<Level>(&include_str!("../resources/levels/level_1.toml")).unwrap());
levels.insert("level_2".to_string(), toml::from_str::<Level>(&include_str!("../resources/levels/level_2.toml")).unwrap());
levels.insert("start".to_string(), toml::from_str::<Level>(&include_str!("../resources/levels/start.toml")).unwrap());
true
}
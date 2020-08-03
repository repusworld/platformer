fn main() {
    if let Ok(p) = std::fs::read_dir("resources/levels").map(|d| d.flatten().map(|f| f.path())) {
        let p = p
            .filter(|p| match p.extension() {
                Some(s) => s.to_string_lossy().to_string() == "toml",
                _ => false,
            })
            .map(|mut p| {
                let path = format!("{}", p.to_string_lossy().to_string());
                p.set_extension("");
                let name = p
                    .file_name()
                    .expect("File name is not valid utf-8!")
                    .to_string_lossy()
                    .to_string();

                format!("levels.insert(\"{}\".to_string(), toml::from_str::<Level>(&include_str!(\"../{}\")).unwrap());", name, path.replace("\\", "/"))
            });

        let mut v = vec![
            "use std::collections::HashMap;".to_string(),
            "use crate::level::Level;".to_string(),
            "pub fn add_default_levels(levels: &mut HashMap<String, Level>) -> bool {".to_string(),
        ];

        for s in p {
            v.push(s.clone());
        }

        v.push("true".to_string());
        v.push("}".to_string());
        std::fs::write("generated/default_levels.rs", format!("{}", v.join("\n"))).unwrap();
    } else {
        let mut v = vec![
            "use std::collections::HashMap;".to_string(),
            "use crate::level::Level;".to_string(),
            "pub fn add_default_levels(levels: &mut HashMap<String, Level>) -> bool {".to_string(),
        ];

        v.push("false".to_string());
        v.push("}".to_string());
        std::fs::write("generated/default_levels.rs", format!("{}", v.join("\n"))).unwrap();
    }
}

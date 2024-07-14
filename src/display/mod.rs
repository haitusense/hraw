pub trait Indent { fn indent(&self, level:usize) -> String; }
impl Indent for String {
  fn indent(&self, level:usize) -> String {
    self.lines()
      .map(|line| format!("{}{}", " ".repeat(level), line))
      .collect::<Vec<_>>().join("\n")
  }
}

pub fn obj_to_json<T: serde::ser::Serialize>(src:T) -> anyhow::Result<String> {
  let value = serde_json::to_value(src)?;
  let json = serde_json::to_string_pretty(&value)?;
  Ok(json)
}

pub fn obj_to_yaml<T: serde::ser::Serialize>(src:T) -> anyhow::Result<String> {
  let value = serde_json::to_value(src)?;
  let yaml = serde_yaml::to_string(&value)?;
  Ok(yaml)
}

  // println!("{}", "subpath".green());
  // let subpath: std::collections::BTreeMap<usize,String> = subpath.into_iter()
  //   .map(|(k, v)| (k, format!("{} {}", v.kind, v.path))).collect();
  // println!("{}", serde_yaml::to_string(&subpath)?.indent(2));
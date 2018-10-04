use std::fmt;
use std::io::Write;

use error::{Result, ResultLiquidChainExt};
use value::Index;
use value::Path;

use super::Context;
use super::Renderable;

/// A `Value` reference.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variable {
    path: Path,
}

impl Variable {
    /// Create a `Value` reference.
    pub fn new<I: Into<Index>>(value: I) -> Self {
        let path = Path::with_index(value);
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Extend<Index> for Variable {
    fn extend<T: IntoIterator<Item = Index>>(&mut self, iter: T) {
        self.path.extend(iter);
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl Renderable for Variable {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = context.stack().get(&self.path)?;
        write!(writer, "{}", value).chain("Failed to render")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_yaml;

    use super::super::ContextBuilder;
    use super::*;
    use value::Object;

    #[test]
    fn identifier_path_array_index() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test"]
"#,
        ).unwrap();
        let mut actual = Variable::new("test_a");
        let index = vec![Index::with_index(0)];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "test".to_owned());
    }

    #[test]
    fn identifier_path_array_index_negative() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test1", "test2"]
"#,
        ).unwrap();
        let mut actual = Variable::new("test_a");
        let index = vec![Index::with_index(-1)];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "test2".to_owned());
    }

    #[test]
    fn identifier_path_object() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a:
  - test_h: 5
"#,
        ).unwrap();
        let mut actual = Variable::new("test_a");
        let index = vec![Index::with_index(0), Index::with_key("test_h")];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "5".to_owned());
    }
}

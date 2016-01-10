use std::collections::HashMap;
use value::Value;


#[derive(Default)]
pub struct Context<'a>{
    values : HashMap<String, Value>,
    pub filters : HashMap<String, Box<Fn(&str, &Vec<Value>) -> String + 'a>>
}

impl<'a> Context<'a> {
    /// Creates a new, empty rendering context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use liquid::Value;
    /// # use liquid::Context;
    /// let mut ctx = Context::new();
    /// assert_eq!(ctx.get_val("test"), None);
    /// ```
    pub fn new() -> Context<'a> {
        Context::with_values_and_filters(HashMap::new(), HashMap::new())
    }

    pub fn with_values(values: HashMap<String, Value>) -> Context<'a> {
        Context::with_values_and_filters(values, HashMap::new())
    }

    pub fn with_filters(filters: HashMap<String, Box<Fn(&str, &Vec<Value>) -> String + 'a>>) -> Context<'a> {
        Context::with_values_and_filters(HashMap::new(), filters)
    }

    pub fn with_values_and_filters(values: HashMap<String, Value>, filters: HashMap<String, Box<Fn(&str, &Vec<Value>) -> String + 'a>>) -> Context<'a> {
        Context {
            values: values,
            filters: filters
        }
    }

    /// Gets a value from the rendering context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use liquid::Value;
    /// # use liquid::Context;
    /// let mut ctx = Context::new();
    /// ctx.set_val("test", Value::Num(42f32));
    /// assert_eq!(ctx.get_val("test").unwrap(), &Value::Num(42f32));
    /// ```
    pub fn get_val(&self, name: &str) -> Option<&Value> {
        let mut it = name.split('.');
        let mut ret = self.values.get(it.next().unwrap_or(""));
        for id in it{
            match ret {
                Some(&Value::Object(ref x)) => ret = x.get(id),
                _ => return None,
            }
        }
        ret
    }

    /// Sets a value to the rendering context.
    /// Note that it needs to be wrapped in a liquid::Value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use liquid::Value;
    /// # use liquid::Context;
    /// let mut ctx = Context::new();
    /// ctx.set_val("test", Value::Num(42f32));
    /// assert_eq!(ctx.get_val("test").unwrap(), &Value::Num(42f32));
    /// ```
    pub fn set_val(&mut self, name: &str, val: Value) -> Option<Value> {
        self.values.insert(name.to_string(), val)
    }
}

#[test]
fn test_get_val() {
    let mut ctx = Context::new();
    let mut post = HashMap::new();
    post.insert("number".to_string(), Value::Num(42f32));
    ctx.set_val("post", Value::Object(post));
    assert_eq!(ctx.get_val("post.number").unwrap(), &Value::Num(42f32));
}



use value::Value;
use value::Value::*;

pub fn size(input : &Value, _args: &Vec<Value>) -> String {
    match input {
        &Str(ref s) => s.len().to_string(),
        &Array(ref x) => x.len().to_string(),
        &Object(ref x) => x.len().to_string(),
        _ => "Unknown length".to_string()
    }
}

pub fn upcase(input: &Value, _args: &Vec<Value>) -> String {
    match input {
        &Str(ref s) => s.to_uppercase(),
        _ => input.to_string()
    }
}

pub fn minus(input: &Value, args: &Vec<Value>) -> String {

    let num = match input {
        &Num(n) => n,
        _ => return input.to_string()
    };
    match args.first() {
        Some(&Num(x)) => (num - x).to_string(),
        _ => num.to_string()
    }
}

pub fn replace(input: &Value, args: &Vec<Value>) -> String {
    match input {
        &Str(ref x) => {
            let arg1 = match &args[0] {
                &Str(ref a) => a, _ => return input.to_string()
            };
            let arg2 = match &args[1] {
                &Str(ref a) => a, _ => return input.to_string()
            };
            x.replace(arg1, arg2)
        },
        _ => input.to_string()
    }
}

macro_rules! unit {
    ( $a:ident, $b:expr ) => {{
        unit!($a, $b, &vec![])
    }};
    ( $a:ident, $b:expr , $c:expr) => {{
        $a(&$b, $c)
    }};
}

macro_rules! tos {
    ( $a:expr ) => {{
        Str($a.to_string())
    }};
}

#[test]
fn unit_size(){
    assert_eq!(unit!(size, tos!("abc")), "3");
    assert_eq!(unit!(size, tos!("this has 22 characters")), "22");
}

#[test]
fn unit_upcase() {
    assert_eq!(unit!(upcase, tos!("abc")), "ABC");
    assert_eq!(unit!(upcase, tos!("Hello World 21")), "HELLO WORLD 21");
}

#[test]
fn unit_minus() {
    assert_eq!(unit!(minus, Num(2f32), &vec![Num(1f32)]), "1");
    assert_eq!(unit!(minus, Num(21.5), &vec![Num(1.25)]), "20.25");
    assert_eq!(unit!(minus, tos!("invalid"), &vec![Num(1.25)]), "invalid");
    assert_eq!(unit!(minus, Num(25f32)), "25");
}

#[test]
fn unit_replace() {
    assert_eq!( unit!(replace, tos!("barbar"), &vec![tos!("bar"), tos!("foo")]), "foofoo" );
}

#[macro_use]
extern crate difference;
extern crate liquid;

use std::fs::File;
use std::io::Read;
use std::thread;
use std::sync::Arc;
use liquid::*;

#[test]
pub fn pass_between_threads() {
    let input_file = "tests/fixtures/input/example.txt";
    let options: LiquidOptions = Default::default();
    let template = parse_file(&input_file, options).unwrap();
    let mut v = Vec::new();

    v.push((Value::Num(5f32), Value::Num(6f32)));
    v.push((Value::Num(20f32), Value::Num(10f32)));

    // Start threads
    let mut handles = Vec::new();
    let mut counter = 0;
    let template = Arc::new(template);
    for (num1, num2) in v {
        let template = template.clone();
        counter += 1;
        handles.push(thread::spawn(move || {
            let mut context = Context::new();
            context.set_val("num", num1);
            context.set_val("numTwo", num2);

            let output = template.render(&mut context).unwrap();

            let output_file = format!("tests/fixtures/output/example_mt{}.txt", counter);
            let mut comp = String::new();
            File::open(output_file).unwrap().read_to_string(&mut comp).unwrap();

            assert_diff!(&comp, &output.unwrap(), " ", 0);
        }));
    }

    // Wait for threads to finish
    handles.into_iter().map(|h| h.join()).last();
}

#[macro_use]
extern crate difference;
extern crate liquid;
extern crate serde_yaml;

use std::fs::File;
use std::io::Read;
use std::thread;
use std::sync::Arc;

#[test]
pub fn pass_between_threads() {
    let input_file = "tests/fixtures/input/example.txt";
    let template = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .parse_file(&input_file)
        .unwrap();
    let template = Arc::new(template);

    // Start threads
    let mut handles = Vec::new();
    let v = vec![(5f32, 6f32), (20f32, 10f32)];
    for (counter, (num1, num2)) in v.into_iter().enumerate() {
        let template = Arc::clone(&template);
        let output_file = format!("tests/fixtures/output/example_mt{}.txt", counter + 1);
        handles.push(thread::spawn(move || {
            let globals: liquid::Object = serde_yaml::from_str(&format!(
                r#"
num: {}
numTwo: {}
"#,
                num1,
                num2
            )).unwrap();
            let output = template.render(&globals).unwrap();

            let mut comp = String::new();
            File::open(&output_file)
                .expect(&format!("Expected output file does not exist: {}", output_file))
                .read_to_string(&mut comp)
                .expect(&format!("Failed to read file: {}", output_file));

            assert_diff!(&comp, &output, " ", 0);
        }));
    }

    // Wait for threads to finish
    handles.into_iter().map(|h| h.join()).last();
}

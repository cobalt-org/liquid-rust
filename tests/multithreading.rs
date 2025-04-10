use std::sync::Arc;
use std::thread;

use snapbox::assert_data_eq;

#[test]
pub fn pass_between_threads() {
    let input_file = "tests/fixtures/input/example.txt";
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse_file(input_file)
        .unwrap();
    let template = Arc::new(template);

    // Start threads
    let mut handles = Vec::new();
    let v = vec![(5f64, 6f64), (20f64, 10f64)];
    for (counter, (num1, num2)) in v.into_iter().enumerate() {
        let template = Arc::clone(&template);
        let output_file = std::path::PathBuf::from(format!(
            "tests/fixtures/output/example_mt{}.txt",
            counter + 1
        ));
        handles.push(thread::spawn(move || {
            let globals = liquid::object!({
                "num": num1,
                "numTwo": num2,
            });
            let output = template.render(&globals).unwrap();

            assert_data_eq!(output, snapbox::Data::read_from(&output_file, None).raw());
        }));
    }

    // Wait for threads to finish
    handles.into_iter().map(|h| h.join()).next_back();
}

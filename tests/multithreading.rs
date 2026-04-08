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
    for handle in handles {
        handle.join().expect("thread render should succeed");
    }
}

#[test]
pub fn shared_template_remains_correct_across_many_concurrent_renders() {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("{{ label }}:{{ value }}")
        .unwrap();
    let template = Arc::new(template);

    let mut handles = Vec::new();
    for worker in 0..8 {
        let template = Arc::clone(&template);
        handles.push(thread::spawn(move || {
            for iteration in 0..25 {
                let globals = liquid::object!({
                    "label": format!("worker-{worker}"),
                    "value": iteration,
                });
                let rendered = template.render(&globals).unwrap();
                assert_eq!(rendered, format!("worker-{worker}:{iteration}"));
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread render should succeed");
    }
}

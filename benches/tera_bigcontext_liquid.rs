#![feature(test)]

extern crate test;

use liquid;

#[bench]
fn bench_big_loop_big_object(b: &mut test::Bencher) {
    const NUM_OBJECTS: usize = 100;
    let objects: Vec<_> = (0..NUM_OBJECTS).map(|i| {
        let data_wrapper= liquid::object!({
            "i": (i as i32),
            "v": "Meta
Before we get to the details, two important notes about the ownership system.
Rust has a focus on safety and speed. It accomplishes these goals through many ‘zero-cost abstractions’, which means that in Rust, abstractions cost as little as possible in order to make them work. The ownership system is a prime example of a zero cost abstraction. All of the analysis we’ll talk about in this guide is done at compile time. You do not pay any run-time cost for any of these features.
However, this system does have a certain cost: learning curve. Many new users to Rust experience something we like to call ‘fighting with the borrow checker’, where the Rust compiler refuses to compile a program that the author thinks is valid. This often happens because the programmer’s mental model of how ownership should work doesn’t match the actual rules that Rust implements. You probably will experience similar things at first. There is good news, however: more experienced Rust developers report that once they work with the rules of the ownership system for a period of time, they fight the borrow checker less and less.
With that in mind, let’s learn about borrowing.",
        });
        liquid::object!({
            "field_a": data_wrapper.clone(),
            "field_b": data_wrapper.clone(),
            "field_c": data_wrapper.clone(),
            "field_d": data_wrapper.clone(),
            "field_e": data_wrapper.clone(),
            "field_f": data_wrapper.clone(),
        })
    }).collect();
    let data = liquid::object!({
        "objects": objects,
    });

    let parser = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap();
    let template = parser
        .parse(
            "
{%- for object in objects -%}
{{ object.field_a.i }}
{%- if object.field_a.i > 2 -%}
{%- break -%}
{%- endif -%}
{%- endfor -%}
",
        )
        .expect("Benchmark template parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

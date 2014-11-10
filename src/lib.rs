#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
use regex::Regex;

static SPLIT : Regex = regex!("(\\{%.*?%\\})|(\\{\\{.*?\\}\\})");

fn split_blocks(text: &str) -> Vec<String>{
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in SPLIT.find_iter(text) {
        match text.slice(current, begin){
            "" => {}
            t => tokens.push(t.to_string())
        };
        tokens.push(text.slice(begin, end).to_string());
        current = end;
    }
    match text.slice(current, text.len()){
        "" => {}
        t => tokens.push(t.to_string())
    };
    tokens
}

#[test]
fn test_split_blocks() {
    assert_eq!(split_blocks("asdlkjfn\n{{askdljfbalkjsdbf}} asdjlfb"),
                vec!["asdlkjfn\n".to_string(), "{{askdljfbalkjsdbf}}".to_string(), " asdjlfb".to_string()]);
}

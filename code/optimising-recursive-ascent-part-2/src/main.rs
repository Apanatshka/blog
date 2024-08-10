use optimizing_directly_executable_lr_parsers::parser;

fn main() {
    let sample_input = "a+a*(a+a)*a";
    println!("parse_reverse_goto");
    parser::parse_reverse_goto(&mut sample_input.chars().peekable()).expect("sample_input should parse_reverse_goto just fine");
    println!("parse_asc_desc");
    parser::parse_reverse_goto(&mut sample_input.chars().peekable()).expect("sample_input should parse_asc_desc just fine");
    println!("parse_push_first");
    parser::parse_push_first(&mut sample_input.chars().peekable()).expect("sample_input should parse_push_first just fine");
    println!("parse_minpush");
    parser::parse_minpush(&mut sample_input.chars().peekable()).expect("sample_input should parse_minpush just fine");
    println!("parse_inline1");
    parser::parse_inline1(&mut sample_input.chars().peekable()).expect("sample_input should parse_inline1 just fine");
    println!("parse_inline2");
    parser::parse_inline2(&mut sample_input.chars().peekable()).expect("sample_input should parse_inline2 just fine");
    println!("parse_single_input_next1");
    parser::parse_single_input_next1(&mut sample_input.chars().peekable()).expect("sample_input should parse_single_input_next1 just fine");
    println!("parse_single_input_next");
    parser::parse_single_input_next(&mut sample_input.chars().peekable()).expect("sample_input should parse_single_input_next just fine");
}

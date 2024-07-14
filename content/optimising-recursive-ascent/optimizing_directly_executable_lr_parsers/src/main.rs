use optimizing_directly_executable_lr_parsers::paper;

fn main() {
    let sample_input = "a+a*(a+a)*a";
    println!("parse");
    paper::parse(&mut sample_input.chars().peekable()).expect("sample_input should parse just fine");
    println!("parse_reverse_goto");
    paper::parse_reverse_goto(&mut sample_input.chars().peekable()).expect("sample_input should parse_reverse_goto just fine");
    println!("parse_chain_elim");
    paper::parse_chain_elim(&mut sample_input.chars().peekable()).expect("sample_input should parse_chain_elim just fine");
    println!("parse_minpush");
    paper::parse_minpush(&mut sample_input.chars().peekable()).expect("sample_input should parse_minpush just fine");
    println!("parse_max_inline");
    paper::parse_max_inline(&mut sample_input.chars().peekable()).expect("sample_input should parse_max_inline just fine");
}

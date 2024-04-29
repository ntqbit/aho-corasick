use aho_corasick::Automation;

fn main() {
    let automation = Automation::build(["CAN", "AN", "A", "she", "he", "hers"].into_iter());

    #[cfg(feature = "dot")]
    println!("{}", automation.dump().to_dot().unwrap());

    let text = "he and she CAN CAR an herb";

    let mut search = automation.search();
    for (i, c) in text.chars().enumerate() {
        let outputs = search.next(&c);
        if !outputs.is_empty() {
            println!("i={}: {:?}", i, outputs);
        }
    }
}

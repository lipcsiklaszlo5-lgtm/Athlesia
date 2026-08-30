use athlesia::{Encoder, RelationalStructure};

fn main() {
    let encoder = Encoder::new();
    let values = [847, 13, 847, 13, 999];

    let sequence = encoder.encode(&values);
    let relations = RelationalStructure::from_sequence(&sequence);

    println!("Athlesia structural relation layer");
    println!("roles={:?}", sequence.roles());
    println!("relation_count={}", relations.relation_count());

    for relation in relations.relations() {
        println!(
            "relation: {:?}({}, {})",
            relation.kind(),
            relation.left(),
            relation.right()
        );
    }
}

use place_macros::*;

fn main() {
    println!("=== 1. Canonical Complex Expression Desugaring ===");
    println!("Expression: (*place.field[42])[8].other_field\n");
    let summary = desugar_place!((*place.field[42])[8].other_field);
    println!("{}\n", summary);

    println!("=== 2. Simple Field Access Desugaring ===");
    let simple_summary = desugar_place!(my_struct.my_field);
    println!("{}\n", simple_summary);

    println!("=== 3. Deref and Indexing Desugaring ===");
    let deref_summary = desugar_place!((*ptr)[10]);
    println!("{}\n", deref_summary);
}

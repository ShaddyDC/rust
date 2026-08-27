use place_macros::*;

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(" )", ")")
}

#[test]
fn test_desugar_path_string() {
    let s = desugar_raw_handle!(my_var);
    assert_eq!(normalize(s), "LocalHandle::new(&raw mut my_var)");

    let s_const = desugar_raw_handle!(const my_var);
    assert_eq!(normalize(s_const), "LocalHandle::new(&raw const my_var)");

    let s_mut = desugar_raw_handle!(mut my_var);
    assert_eq!(normalize(s_mut), "LocalHandle::new(&raw mut my_var)");
}

#[test]
fn test_desugar_deref_string() {
    let s = desugar_raw_handle!(*ptr);
    assert_eq!(normalize(s), "DerefPlace::deref_place(LocalHandle::new(&raw mut ptr))");
}

#[test]
fn test_desugar_field_string() {
    let s = desugar_raw_handle!(place.field);
    assert_eq!(
        normalize(s),
        concat!(
            "ProjectPlace::<field_of!(typeof(place), field)>::project_place(",
            "LocalHandle::new(&raw mut place), ",
            "<field_of!(typeof(place), field)>::default())"
        )
    );
}

#[test]
fn test_desugar_index_string() {
    let s = desugar_raw_handle!(arr[42]);
    assert_eq!(normalize(s), "IndexPlace::index_place(LocalHandle::new(&raw mut arr), 42)");
}

#[test]
fn test_desugar_type_of_string() {
    let s_path = desugar_type_of!(my_var);
    assert_eq!(normalize(s_path), "typeof(my_var)");

    let s_deref = desugar_type_of!(*ptr);
    assert_eq!(normalize(s_deref), "<typeof(ptr) as PlaceProxy>::Target");

    let s_field = desugar_type_of!(place.field);
    assert_eq!(normalize(s_field), "<field_of!(typeof(place), field) as Subplace>::Target");

    let s_index = desugar_type_of!(arr[42]);
    assert_eq!(normalize(s_index), "<typeof(arr) as Indexable<typeof(42)>>");
}

#[test]
fn test_desugar_complex_expression_from_doc() {
    // The doc's exact canonical example: (*place.field[42])[8].other_field
    let s = desugar_raw_handle!((*place.field[42])[8].other_field);
    println!("=== Desugared raw_handle! ===\n{}\n", s);

    assert!(s.contains("other_field"), "Missing other_field: {}", s);
    assert!(s.contains("IndexPlace::index_place"), "Missing IndexPlace: {}", s);
    assert!(s.contains("DerefPlace::deref_place"), "Missing DerefPlace: {}", s);
    assert!(s.contains("LocalHandle::new(&raw mut place)"), "Missing LocalHandle: {}", s);

    let ty_s = desugar_type_of!((*place.field[42])[8].other_field);
    println!("=== Desugared type_of! ===\n{}\n", ty_s);
    assert!(ty_s.contains("PlaceProxy"), "Missing PlaceProxy: {}", ty_s);
    assert!(ty_s.contains("Indexable"), "Missing Indexable: {}", ty_s);
    assert!(ty_s.contains("Subplace"), "Missing Subplace: {}", ty_s);

    let full_summary = desugar_place!((*place.field[42])[8].other_field);
    println!("=== desugar_place! ===\n{}\n", full_summary);
    assert!(full_summary.contains("Place Expression:"));
    assert!(full_summary.contains("Desugared Handle:"));
    assert!(full_summary.contains("Desugared Type:"));
}

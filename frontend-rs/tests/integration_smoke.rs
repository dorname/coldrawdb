use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn smoke() {
    assert_eq!(2 + 2, 4);
}
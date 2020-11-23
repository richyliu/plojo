use plojo::{StandardTranslator, StandardTranslatorConfig, Translator};

#[test]
fn can_make_translator() {
    let raw_dict = r#"
        {
            "H-L": "hello"
        }
    "#
    .to_string();
    // should not panic
    let _ = StandardTranslator::new(StandardTranslatorConfig::new(raw_dict, vec![]))
        .expect("Unable to create translator");
}

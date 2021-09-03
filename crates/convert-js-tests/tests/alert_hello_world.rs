use convert_js::convert_js;

#[test]
fn test_dynamic() {
    let converted = convert_js("const { alert } = self; alert('Hello, World')").unwrap();
    assert_eq!(converted, r#""use strict";(0,self.alert)("Hello, World");"#);
}

#[test]
fn test_static() {
    let converted = convert_js!("const { alert } = self; alert('Hello, World')");
    assert_eq!(converted, r#""use strict";(0,self.alert)("Hello, World");"#);
}

#[test]
#[should_panic]
fn test_dynamic_typo() {
    convert_js("const { alert } = self; alert('Hello, World'").unwrap();
}

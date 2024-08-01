#[test]
fn ready() {
    println!("it works!")
}


#[test]
fn test_uuid() {
    let s = r#"window.QRLogin.code = 200; window.QRLogin.uuid = "Icx9GPaOow==""#;
    let uuid = &s[43..52];
    println!("{uuid}")
}
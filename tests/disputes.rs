use test_accounting::process_csv;

#[test]
fn chargeback_dispute() {
    let input = r#"type,client,tx,amount
deposit,14,1,57097.49
dispute,14,2,16397.12
chargeback,14,2,
deposit,14,1,57097.49
"#;
    let expected_output = r#"client,available,held,locked,total
14,40700.37,16397.12,true,57097.49
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

#[test]
fn unresolved_dispute() {
    let input = r#"type,client,tx,amount
deposit,14,1,57097.49
dispute,14,2,16397.12
deposit,14,1,57097.49
"#;
    let expected_output = r#"client,available,held,locked,total
14,97797.86,16397.12,false,114194.98
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

#[test]
fn resolved_dispute() {
    let input = r#"type,client,tx,amount
deposit,14,1,57097.49
dispute,14,2,16397.12
resolve,14,2,
deposit,14,1,57097.49
"#;
    let expected_output = r#"client,available,held,locked,total
14,114194.98,0,false,114194.98
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

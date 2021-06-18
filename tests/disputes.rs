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

#[test]
fn provided_case() {
    let input = r#"type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
"#;
    let expected_output = r#"client,available,held,locked,total
1,1.5,0,false,1.5
2,2,0,false,2
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

#[test]
fn no_newline_eof() {
    let input = r#"type,client,tx,amount
deposit,1,1,1.0"#;
    let expected_output = r#"client,available,held,locked,total
1,1,0,false,1
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

#[test]
fn newline_start_input() {
    let input = r#"

type,client,tx,amount
deposit,1,1,1.0
"#;
    let expected_output = r#"client,available,held,locked,total
1,1,0,false,1
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

#[test]
fn weird_spacing() {
    let input = r#"

type,		 	client, tx,       amount	 	 	
			deposit	,1  	, 	1 	,					 	 	 	 	 	1.0 	 	"#;
    let expected_output = r#"client,available,held,locked,total
1,1,0,false,1
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}

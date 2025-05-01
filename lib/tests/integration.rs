use well_field::{FieldEnum, Fielded, SetFieldError};

#[derive(FieldEnum)]
#[field(derive(Debug))]
struct MyStruct {
    num: f32,
    count: usize,
    name: String,
    nested: InnerStruct,

    #[allow(dead_code)]
    #[field(skip)]
    skip_me: usize,
}

#[derive(FieldEnum)]
#[field(derive(Debug))]
struct InnerStruct {
    label: String,
    prop: f32,
}

#[test]
fn test_set_field() {
    let mut s = MyStruct {
        num: 1.,
        count: 3,
        name: "hello world".into(),
        nested: InnerStruct {
            label: "hi".into(),
            prop: 123.,
        },
        skip_me: 100,
    };

    s.set_field(MyStructField::Num, 789.).unwrap();
    assert_eq!(s.num, 789.);
    assert!(s.set_field(MyStructField::Num, 1).is_err());

    s.set_field(MyStructField::Name, "abc".to_string()).unwrap();
    assert_eq!(s.name.as_str(), "abc");
    assert!(s.set_field(MyStructField::Name, 1).is_err());

    // TODO: ideally we can just pass in 456. here without needing
    // to wrap it in `InnerStructValue::from`, but I'm not sure how to accomplish this.
    s.set_field(
        MyStructField::Nested(InnerStructField::Prop),
        InnerStructValue::from(456.),
    )
    .unwrap();
    assert_eq!(s.nested.prop, 456.);
    assert!(
        s.set_field(MyStructField::Nested(InnerStructField::Prop), 1)
            .is_err()
    );

    println!("{:?}", MyStructField::Num);
}

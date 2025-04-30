# well-field

A small crate which allows for updating a struct's fields individually in a more abstract way. This provides a way of keeping "diffs" or partial updates for a struct in a more composable way. In contrast to a crate like [`partially`](https://crates.io/crates/partially) or [`gremlin`](https://github.com/frnsys/gremlin), which model partials as an entire copy of the struct with fields wrapped in `Option`, `well-field` lets you mix-and-match field values so that they can be combinatorial:

```rust
PartialStruct {
  a: Option<A>,
  b: Option<B>,
  c: Option<C>,
}

// vs

[(StructField::A, A::0), ..]
[(StructField::B, B::0), ..]
```


# Usage

```rust
use well_field::{FieldEnum, Fielded, SetFieldError};

#[derive(FieldEnum)]
struct MyStruct {
    num: f32,
    count: usize,
    name: String,
    nested: InnerStruct,

    #[field(skip)]
    skip_me: usize,
}

#[derive(FieldEnum)]
struct InnerStruct {
    label: String,
    prop: f32,
}

fn main() {
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
    s.set_field(MyStructField::Name, "abc".to_string()).unwrap();
    s.set_field(
        MyStructField::Nested(InnerStructField::Prop),
        InnerStructValue::from(456.),
    )
    .unwrap();
}
```

---

_See "[well-field](https://en.wikipedia.org/wiki/Well-field_system)"._

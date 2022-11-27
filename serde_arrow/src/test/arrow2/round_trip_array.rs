//! Test round trips on the individual array level without the out records
//!

use std::{collections::BTreeMap, fmt::Debug};

use arrow2::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};

use crate::{
    arrow2::{sinks::build_dynamic_array_builder, sources::build_dynamic_source},
    base::{deserialize_from_source, serialize_into_sink, Event},
    generic::{
        schema::{FieldBuilder, Tracer},
        sinks::ArrayBuilder,
    },
    test::utils::collect_events,
    Strategy, STRATEGY_KEY,
};

macro_rules! test_round_trip {
    (
        test_name = $test_name:ident,
        field = $field:expr,
        ty = $ty:ty,
        values = $values:expr,
    ) => {
        #[test]
        fn $test_name() {
            let items: &[$ty] = &$values;

            let field = $field;

            let mut tracer = Tracer::new();
            for item in items {
                serialize_into_sink(&mut tracer, &item).unwrap();
            }
            let res_field = tracer.to_field("value").unwrap();
            assert_eq!(res_field, field);

            let mut sink = build_dynamic_array_builder(&field).unwrap();
            for item in items {
                serialize_into_sink(&mut sink, &item).unwrap();
            }
            let array = sink.into_array().unwrap();

            let source = build_dynamic_source(&field, array.as_ref()).unwrap();
            let events = collect_events(source).unwrap();

            // add the outer sequence
            let events = {
                let mut events = events;
                events.insert(0, Event::StartSequence);
                events.push(Event::EndSequence);
                events
            };

            let res_items: Vec<$ty> = deserialize_from_source(&events).unwrap();
            assert_eq!(res_items, items);
        }
    };
}

test_round_trip!(
    test_name = primitive_i8,
    field = Field::new("value", DataType::Int8, false),
    ty = i8,
    values = [0, 1, 2],
);
test_round_trip!(
    test_name = nullable_i8,
    field = Field::new("value", DataType::Int8, true),
    ty = Option<i8>,
    values = [Some(0), None, Some(2)],
);
test_round_trip!(
    test_name = nullable_i8_only_some,
    field = Field::new("value", DataType::Int8, true),
    ty = Option<i8>,
    values = [Some(0), Some(2)],
);

test_round_trip!(
    test_name = primitive_f32,
    field = Field::new("value", DataType::Float32, false),
    ty = f32,
    values = [0.0, 1.0, 2.0],
);
test_round_trip!(
    test_name = nullable_f32,
    field = Field::new("value", DataType::Float32, true),
    ty = Option<f32>,
    values = [Some(0.0), None, Some(2.0)],
);
test_round_trip!(
    test_name = nullable_f32_only_some,
    field = Field::new("value", DataType::Float32, true),
    ty = Option<f32>,
    values = [Some(0.0), Some(2.0)],
);

test_round_trip!(
    test_name = primitive_bool,
    field = Field::new("value", DataType::Boolean, false),
    ty = bool,
    values = [true, false, true],
);
test_round_trip!(
    test_name = nullable_bool,
    field = Field::new("value", DataType::Boolean, true),
    ty = Option<bool>,
    values = [Some(true), None, Some(false)],
);
test_round_trip!(
    test_name = nullable_bool_only_some,
    field = Field::new("value", DataType::Boolean, true),
    ty = Option<bool>,
    values = [Some(true), Some(false)],
);

test_round_trip!(
    test_name = vec_bool,
    field = Field::new(
        "value",
        DataType::LargeList(Box::new(Field::new("element", DataType::Boolean, false))),
        false,
    ),
    ty = Vec<bool>,
    values = [vec![true, false], vec![], vec![false]],
);
test_round_trip!(
    test_name = nullable_vec_bool,
    field = Field::new("value", DataType::LargeList(Box::new(Field::new("element", DataType::Boolean, false))), true),
    ty = Option<Vec<bool>>,
    values = [Some(vec![true, false]), Some(vec![]), None],
);
test_round_trip!(
    test_name = vec_nullable_bool,
    field = Field::new("value", DataType::LargeList(Box::new(Field::new("element", DataType::Boolean, true))), false),
    ty = Vec<Option<bool>>,
    values = [vec![Some(true), Some(false)], vec![], vec![None, Some(false)]],
);

test_round_trip!(
    test_name = struct_nullable,
    field = Field::new("value",DataType::Struct(vec![
        Field::new("a", DataType::Boolean, false),
        Field::new("b", DataType::Int64, false),
        Field::new("c", DataType::Null, true),
        Field::new("d", DataType::LargeUtf8, false),
    ]), true),
    ty = Option<Struct>,
    values = [
        Some(Struct {
            a: true,
            b: 42,
            c: (),
            d: String::from("hello"),
        }),
        None,
        Some(Struct {
            a: false,
            b: 13,
            c: (),
            d: String::from("world"),
        }),
    ],
);

test_round_trip!(
    test_name = struct_nullable_nested,
    field = Field::new("value",DataType::Struct(vec![
        Field::new("inner", DataType::Struct(vec![
            Field::new("a", DataType::Boolean, false),
            Field::new("b", DataType::Int64, false),
            Field::new("c", DataType::Null, true),
            Field::new("d", DataType::LargeUtf8, false),
        ]), false),
    ]),true),
    ty = Option<Outer>,
    values = [
        Some(Outer {
            inner: Struct {
            a: true,
            b: 42,
            c: (),
            d: String::from("hello"),
        }}),
        None,
        Some(Outer {inner: Struct {
            a: false,
            b: 13,
            c: (),
            d: String::from("world"),
        }}),
    ],
);

test_round_trip!(
    test_name = struct_nullable_item,
    field = Field::new(
        "value",
        DataType::Struct(vec![
            Field::new("a", DataType::Boolean, true),
            Field::new("b", DataType::Int64, true),
            Field::new("c", DataType::Null, true),
            Field::new("d", DataType::LargeUtf8, true),
        ]),
        false
    ),
    ty = StructNullable,
    values = [
        StructNullable {
            a: None,
            b: None,
            c: None,
            d: Some(String::from("hello")),
        },
        StructNullable {
            a: Some(true),
            b: Some(42),
            c: None,
            d: None,
        },
    ],
);

test_round_trip!(
    test_name = tuple_nullable,
    field = Field::new("value", DataType::Struct(vec![
        Field::new("0", DataType::Boolean, false),
        Field::new("1", DataType::Int64, false),
    ]), true).with_metadata(strategy_meta(Strategy::Tuple)),
    ty = Option<(bool, i64)>,
    values = [
        Some((true, 21)),
        None,
        Some((false, 42)),
    ],
);

test_round_trip!(
    test_name = tuple_nullable_nested,
    field = Field::new("value", DataType::Struct(vec![
        Field::new("0", DataType::Struct(vec![
                Field::new("0", DataType::Boolean, false),
                Field::new("1", DataType::Int64, false),
            ]), false)
            .with_metadata(strategy_meta(Strategy::Tuple)),
        Field::new("1", DataType::Int64, false),
    ]), true).with_metadata(strategy_meta(Strategy::Tuple)),
    ty = Option<((bool, i64), i64)>,
    values = [
        Some(((true, 21), 7)),
        None,
        Some(((false, 42), 13)),
    ],
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Struct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Struct {
    a: bool,
    b: i64,
    c: (),
    d: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StructNullable {
    a: Option<bool>,
    b: Option<i64>,
    c: Option<()>,
    d: Option<String>,
}

fn strategy_meta(strategy: Strategy) -> BTreeMap<String, String> {
    let mut meta = BTreeMap::new();
    meta.insert(STRATEGY_KEY.to_string(), strategy.to_string());
    meta
}

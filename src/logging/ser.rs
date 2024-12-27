use serde_json::json;
use slog::Key;
use std::fmt;

pub(crate) struct KubewardenLogSerializer {
    data: serde_json::Map<String, serde_json::Value>,
}

impl KubewardenLogSerializer {
    pub fn start() -> Result<Self, slog::Error> {
        Ok(KubewardenLogSerializer {
            data: serde_json::Map::new(),
        })
    }

    pub fn field_serializer(&mut self) -> KubewardenFieldSerializer {
        KubewardenFieldSerializer {
            data: &mut self.data,
        }
    }

    pub fn end(self) -> Result<serde_json::Map<String, serde_json::Value>, slog::Error> {
        Ok(self.data)
    }
}

pub(crate) struct KubewardenFieldSerializer<'a> {
    data: &'a mut serde_json::Map<String, serde_json::Value>,
}

macro_rules! emit_m {
    ($f:ident, $arg:ty) => {
        fn $f(&mut self, key: Key, val: $arg) -> slog::Result {
            self.data.insert(key.into(), val.into());
            Ok(())
        }
    };
}

impl slog::Serializer for KubewardenFieldSerializer<'_> {
    emit_m!(emit_u8, u8);
    emit_m!(emit_i8, i8);
    emit_m!(emit_u16, u16);
    emit_m!(emit_i16, i16);
    emit_m!(emit_usize, usize);
    emit_m!(emit_isize, isize);
    emit_m!(emit_u32, u32);
    emit_m!(emit_i32, i32);
    emit_m!(emit_u64, u64);
    emit_m!(emit_i64, i64);
    emit_m!(emit_f32, f32);
    emit_m!(emit_f64, f64);
    emit_m!(emit_bool, bool);
    emit_m!(emit_str, &str);

    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
        self.data.insert(key.into(), format!("{}", val).into());
        Ok(())
    }

    // Serialize '()' as '0'
    fn emit_unit(&mut self, key: Key) -> slog::Result {
        self.data.insert(key.into(), json!(0));
        Ok(())
    }

    // Serialize 'None' as 'false'
    fn emit_none(&mut self, key: Key) -> slog::Result {
        self.data.insert(key.into(), serde_json::Value::Null);
        Ok(())
    }

    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> slog::Result {
        self.data.insert(key.into(), format!("{}", val).into());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use slog::{o, Record, KV};

    fn do_serializer<S: slog::Serializer>(serializer: &mut S) {
        // rinfo_static and the values passed to Record::new are irrelevant for this test and
        // exist only to fulfill the function arguments
        let rinfo_static = slog::RecordStatic {
            location: &slog::RecordLocation {
                file: "file",
                line: 0,
                column: 0,
                function: "function",
                module: "slog_telegraf::ser::test",
            },
            tag: "slog_tag",
            level: slog::Level::Info,
        };

        o!(
             "int0" => 10 as u8,
             "int1" => -10 as i8,
             "int2" => 10000 as u16,
             "int3" => -10000 as i16,
             "int4" => 2_000_000_000 as u32,
             "int5" => -2_000_000_000 as i32,
             "int6" => 2_000_000_000 as usize,
             "int7" => -2_000_000_000 as isize,
             "int8" => 2_000_000_000_000 as u64,
             "int9" => -2_000_000_000_000 as i64,
             "float0" => 13.2 as f32,
             "float1" => -105.2 as f64,
             "string0" => "foo",
             "string1" => "1.2.1",
             "char0" => 'x',
             "bool0" => true,
             "bool1" => false,
             "unit" => (),
             "none" => Option::<()>::None,
        )
        .serialize(
            &Record::new(
                &rinfo_static,
                &format_args!("msg_{}", "foo"),
                slog::BorrowedKV(&o!("key" => "val")),
            ),
            serializer,
        )
        .unwrap();
    }

    #[test]
    fn test_field_serializer() {
        let mut serializer = KubewardenLogSerializer::start().unwrap();
        let mut field_serializer = serializer.field_serializer();

        do_serializer(&mut field_serializer);

        let data = serializer.end().unwrap();

        let mut expected: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        expected.insert("bool0".into(), json!(true));
        expected.insert("bool1".into(), json!(false));
        expected.insert("char0".into(), json!("x"));
        expected.insert("float0".into(), json!(13.199999809265137));
        expected.insert("float1".into(), json!(-105.2));
        expected.insert("int0".into(), json!(10));
        expected.insert("int1".into(), json!(-10));
        expected.insert("int2".into(), json!(10000));
        expected.insert("int3".into(), json!(-10000));
        expected.insert("int4".into(), json!(2000000000));
        expected.insert("int5".into(), json!(-2000000000));
        expected.insert("int6".into(), json!(2000000000));
        expected.insert("int7".into(), json!(-2000000000));
        expected.insert("int8".into(), json!(2000000000000 as i64));
        expected.insert("int9".into(), json!(-2000000000000 as i64));
        expected.insert("none".into(), serde_json::Value::Null);
        expected.insert("string0".into(), json!("foo"));
        expected.insert("string1".into(), json!("1.2.1"));
        expected.insert("unit".into(), json!(0));

        assert_eq!(data, expected);
    }
}

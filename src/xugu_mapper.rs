use rbatis::table_sync::ColumnMapper;
use rbs::Value;

pub struct XuguTableMapper;

impl ColumnMapper for XuguTableMapper {
    fn driver_type(&self) -> String {
        "xugu".to_string()
    }

    fn get_column_type(&self, column: &str, v: &Value) -> String {
        match v {
            Value::Null => "NULL".to_string(),
            Value::Bool(_) => "BOOLEAN".to_string(),
            Value::I32(_) => "INTEGER".to_string(),
            Value::I64(_) => "BIGINT".to_string(),
            Value::U32(_) => "INTEGER".to_string(),
            Value::U64(_) => "BIGINT".to_string(),
            Value::F32(_) => "FLOAT".to_string(),
            Value::F64(_) => "DOUBLE".to_string(),
            Value::String(v) => {
                if v != "" {
                    v.to_string()
                } else {
                    if column.eq("id") || column.ends_with("_id") || column.starts_with("id_") {
                        return "VARCHAR(50)".to_string();
                    }
                    "VARCHAR(100)".to_string()
                }
            }
            Value::Binary(_) => "BLOB".to_string(),
            Value::Array(_) => "CLOB".to_string(),
            Value::Map(_) => "CLOB".to_string(),
            Value::Ext(t, _v) => match *t {
                "Date" => "DATE".to_string(),
                "DateTime" => "DATETIME".to_string(),
                "Time" => "TIME".to_string(),
                "Timestamp" => "DATETIME".to_string(),
                "Decimal" => "NUMERIC".to_string(),
                "Json" => "JSON".to_string(),
                "Uuid" => "GUID".to_string(),
                _ => "NULL".to_string(),
            },
        }
    }
}

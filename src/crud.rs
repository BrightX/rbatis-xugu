///PySql: gen insert/insert_batch, select_by_map, update_by_map, delete_by_map methods
///
/// 该注解针对虚谷的批量插入的SQL语法，进行重写
///
///```rust
/// use rbs::value;
/// use rbatis::{Error, RBatis, rbdc::db::ExecResult};
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct MockTable{
///    pub id: Option<String>
/// }
/// rbatis_xugu::crud!(MockTable{}); //or crud!(MockTable{},"mock_table");
///
/// //use
/// async fn test_use(rb:&RBatis) -> Result<(),Error>{
///  let table = MockTable{id: Some("1".to_string())};
///  let result:ExecResult = MockTable::insert(rb, &table).await?;
///  let result:ExecResult = MockTable::insert_batch(rb, std::slice::from_ref(&table),10).await?;
///
///  let tables:Vec<MockTable> = MockTable::select_by_map(rb,value!{"id":"1"}).await?;
///  let tables:Vec<MockTable> = MockTable::select_by_map(rb,value!{"id":["1","2","3"]}).await?;
///  let tables:Vec<MockTable> = MockTable::select_by_map(rb,value!{"id":"1", "column": ["id", "name"]}).await?;
///
///  let result:ExecResult = MockTable::update_by_map(rb, &table, value!{"id":"1"}).await?;
///  let result:ExecResult = MockTable::delete_by_map(rb, value!{"id":"1"}).await?;
///  Ok(())
/// }
///
///
/// ```
#[macro_export]
macro_rules! crud {
    ($table:ty{}) => {
        $crate::crud!($table {}, "");
    };
    ($table:ty{},$table_name:expr) => {
        // insert
        impl $table {
            /// batch insert records
            ///
            /// sql: `INSERT INTO table_name (column1, column2, ...) VALUES (value1, value2, ...), (value1, value2, ...), ...`
            pub async fn insert_batch(
                executor: &dyn rbatis::executor::Executor,
                tables: &[$table],
                batch_size: u64,
            ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error> {
                use rbatis::crud_traits::ColumnSet;
                #[rbatis::py_sql(
                    "`insert into ${table_name} `
                    trim ',':
                     bind columns = tables.column_sets():
                     for idx,table in tables:
                      if idx == 0:
                         `(`
                         trim ',':
                           for _,v in columns:
                              ${v},
                         `) VALUES `
                      (
                      trim ',':
                       for _,v in columns:
                         #{table[v]},
                      )
                    "
                )]
                async fn insert_batch(
                    executor: &dyn rbatis::executor::Executor,
                    tables: &[$table],
                    table_name: &str,
                ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error>
                {
                    impled!()
                }
                if tables.is_empty() {
                    return Err(rbatis::rbdc::Error::from(
                        "insert can not insert empty array tables!",
                    ));
                }
                let mut table_name = $table_name.to_string();
                if table_name.is_empty() {
                    #[rbatis::snake_name($table)]
                    fn snake_name() {}
                    table_name = snake_name();
                }
                let mut result = rbatis::rbdc::db::ExecResult {
                    rows_affected: 0,
                    last_insert_id: rbs::Value::Null,
                };
                let ranges =
                    rbatis::plugin::Page::<()>::make_ranges(tables.len() as u64, batch_size);
                for (offset, limit) in ranges {
                    let exec_result = insert_batch(
                        executor,
                        &tables[offset as usize..limit as usize],
                        table_name.as_str(),
                    )
                    .await?;
                    result.rows_affected += exec_result.rows_affected;
                    result.last_insert_id = exec_result.last_insert_id;
                }
                Ok(result)
            }

            /// insert a single record
            ///
            /// sql: `INSERT INTO table_name (column1, column2, ...) VALUES (value1, value2, ...)`
            pub async fn insert(
                executor: &dyn rbatis::executor::Executor,
                table: &$table,
            ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error> {
                <$table>::insert_batch(executor, std::slice::from_ref(table), 1).await
            }
        }
        // select
        impl $table {
            /// select records by condition map.
            /// supports "column" key in condition to select specific columns, e.g. `value!{"col1":"val1", "column": ["col1", "col2"]}`
            ///
            /// sql: `SELECT col1, col2, ... FROM table_name WHERE col1 = ? and col2 in (?, ?, ...)`
            ///
            /// condition map -> where sql:
            /// - `value!{"col1": "val1"}`                       -> `WHERE col1 = 'val1'`
            /// - `value!{"col1": 1, "col2": "val2"}`           -> `WHERE col1 = 1 and col2 = 'val2'`
            /// - `value!{"col1": ["v1", "v2", "v3"]}`          -> `WHERE col1 in ('v1', 'v2', 'v3')`
            /// - `value!{"col1": "val1", "col2": ["a", "b"]}`  -> `WHERE col1 = 'val1' and col2 in ('a', 'b')`
            /// - null values are skipped
            pub async fn select_by_map(
                executor: &dyn rbatis::executor::Executor,
                mut condition: rbs::Value,
            ) -> std::result::Result<Vec<$table>, rbatis::rbdc::Error> {
                use rbatis::crud_traits::ValueOperatorSql;
                // Extract column specification and remove it from condition
                let table_column = {
                    let mut columns = String::new();
                    let mut clean_map = rbs::value::map::ValueMap::with_capacity(condition.len());
                    for (k, v) in condition {
                        match k.as_str() {
                            Some("column") => {
                                columns = match v {
                                    rbs::Value::String(s) => s.clone(),
                                    rbs::Value::Array(arr) => {
                                        let cols: Vec<&str> =
                                            arr.iter().filter_map(|v| v.as_str()).collect();
                                        if cols.is_empty() {
                                            "*".to_string()
                                        } else {
                                            cols.join(", ")
                                        }
                                    }
                                    _ => "*".to_string(),
                                };
                            }
                            _ => {
                                clean_map.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    if columns.is_empty() {
                        columns = "*".to_string();
                    }
                    condition = rbs::Value::Map(clean_map);
                    columns
                };

                #[rbatis::py_sql(
                    "`select ${table_column} from ${table_name}`
           trim end=' where ':
             ` where `
             trim ' and ': for key,item in condition:
                          if item == null:
                             continue:
                          if !item.is_array():
                            ` and ${key.operator_sql()}#{item}`
                          if item.is_array():
                            ` and ${key} in (`
                               trim ',': for _,item_array in item:
                                    #{item_array},
                            `)`
        "
                )]
                async fn select_by_map(
                    executor: &dyn rbatis::executor::Executor,
                    table_name: String,
                    table_column: &str,
                    condition: &rbs::Value,
                ) -> std::result::Result<Vec<$table>, rbatis::rbdc::Error> {
                    for (_, v) in condition {
                        if v.is_array() && v.is_empty() {
                            return Ok(vec![]);
                        }
                    }
                    impled!()
                }
                let mut table_name = $table_name.to_string();
                if table_name.is_empty() {
                    #[rbatis::snake_name($table)]
                    fn snake_name() {}
                    table_name = snake_name();
                }
                select_by_map(executor, table_name, &table_column, &condition).await
            }
        }
        // update
        impl $table {
            /// update records by condition map.
            /// supports "column" key in condition to update specific columns, e.g. `value!{"col1":"val1", "column": ["col2", "col3"]}`
            ///
            /// sql: `UPDATE table_name SET col1 = ?, col2 = ?, ... WHERE col1 = ? and col2 in (?, ?, ...)`
            /// note: skips null fields by default, skips 'id' field always
            ///
            /// condition map -> where sql:
            /// - `value!{"col1": "val1"}`                       -> `WHERE col1 = 'val1'`
            /// - `value!{"col1": 1, "col2": "val2"}`           -> `WHERE col1 = 1 and col2 = 'val2'`
            /// - `value!{"col1": ["v1", "v2", "v3"]}`          -> `WHERE col1 in ('v1', 'v2', 'v3')`
            /// - `value!{"col1": "val1", "col2": ["a", "b"]}`  -> `WHERE col1 = 'val1' and col2 in ('a', 'b')`
            /// - null values are skipped
            pub async fn update_by_map(
                executor: &dyn rbatis::executor::Executor,
                table: &$table,
                mut condition: rbs::Value,
            ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error> {
                use rbatis::crud_traits::{FilterByColumns, ValueOperatorSql};

                // Extract column list for selective updates - implements GitHub issue #591
                let set_columns = {
                    let mut columns = rbs::Value::Null;
                    let mut clean_map = rbs::value::map::ValueMap::with_capacity(condition.len());
                    for (k, v) in condition {
                        match k.as_str() {
                            Some("column") => {
                                columns = match v {
                                    rbs::Value::String(s) => {
                                        rbs::Value::Array(vec![rbs::Value::String(s.clone())])
                                    }
                                    rbs::Value::Array(arr) => {
                                        let filtered_array: Vec<rbs::Value> = arr
                                            .iter()
                                            .filter(|v| v.as_str().is_some())
                                            .cloned()
                                            .collect();
                                        if filtered_array.is_empty() {
                                            rbs::Value::Null
                                        } else {
                                            rbs::Value::Array(filtered_array)
                                        }
                                    }
                                    _ => rbs::Value::Null,
                                };
                            }
                            _ => {
                                clean_map.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    condition = rbs::Value::Map(clean_map);
                    columns
                };
                #[rbatis::py_sql(
                    "`update ${table_name}
                      if skip_null == false:
                        set collection='table',skips='id',skip_null=false:
                      if skip_null == true:
                        set collection='table',skips='id':
                      trim end=' where ':
                       ` where `
                       trim ' and ': for key,item in condition:
                            if item == null:
                               continue:
                            if !item.is_array():
                              ` and ${key.operator_sql()}#{item}`
                            if item.is_array():
                              ` and ${key} in (`
                                 trim ',': for _,item_array in item:
                                      #{item_array},
                              `)`
                    "
                )]
                async fn update_by_map_internal(
                    executor: &dyn rbatis::executor::Executor,
                    table_name: String,
                    table: &rbs::Value,
                    condition: &rbs::Value,
                    skip_null: bool,
                ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error>
                {
                    for (_, v) in condition {
                        if v.is_array() && v.is_empty() {
                            return Ok(rbatis::rbdc::db::ExecResult::default());
                        }
                    }
                    impled!()
                }
                let mut table_name = $table_name.to_string();
                if table_name.is_empty() {
                    #[rbatis::snake_name($table)]
                    fn snake_name() {}
                    table_name = snake_name();
                }
                let table_value = rbs::value!(table);
                let mut skip_null = true;
                let table = if set_columns != rbs::Value::Null {
                    skip_null = false;
                    table_value.filter_by_columns(&set_columns)
                } else {
                    table_value
                };
                update_by_map_internal(executor, table_name, &table, &condition, skip_null).await
            }
        }
        // delete
        impl $table {
            /// delete records by condition map
            ///
            /// sql: `DELETE FROM table_name WHERE col1 = ? and col2 in (?, ?, ...)`
            ///
            /// condition map -> where sql:
            /// - `value!{"col1": "val1"}`                       -> `WHERE col1 = 'val1'`
            /// - `value!{"col1": 1, "col2": "val2"}`           -> `WHERE col1 = 1 and col2 = 'val2'`
            /// - `value!{"col1": ["v1", "v2", "v3"]}`          -> `WHERE col1 in ('v1', 'v2', 'v3')`
            /// - `value!{"col1": "val1", "col2": ["a", "b"]}`  -> `WHERE col1 = 'val1' and col2 in ('a', 'b')`
            /// - null values are skipped
            pub async fn delete_by_map(
                executor: &dyn rbatis::executor::Executor,
                condition: rbs::Value,
            ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error> {
                use rbatis::crud_traits::ValueOperatorSql;
                #[rbatis::py_sql(
                    "`delete from ${table_name}`
           trim end=' where ':
             ` where `
             trim ' and ': for key,item in condition:
                          if item == null:
                             continue:
                          if !item.is_array():
                            ` and ${key.operator_sql()}#{item}`
                          if item.is_array():
                            ` and ${key} in (`
                               trim ',': for _,item_array in item:
                                    #{item_array},
                            `)`
        "
                )]
                async fn delete_by_map(
                    executor: &dyn rbatis::executor::Executor,
                    table_name: String,
                    condition: &rbs::Value,
                ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error>
                {
                    for (_, v) in condition {
                        if v.is_array() && v.is_empty() {
                            return Ok(rbatis::rbdc::db::ExecResult::default());
                        }
                    }
                    impled!()
                }
                let mut table_name = $table_name.to_string();
                if table_name.is_empty() {
                    #[rbatis::snake_name($table)]
                    fn snake_name() {}
                    table_name = snake_name();
                }
                delete_by_map(executor, table_name, &condition).await
            }
        }
    };
}

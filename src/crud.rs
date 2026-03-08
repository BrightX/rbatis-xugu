///PySql: gen select*,update*,insert*,delete* ... methods
///
/// 该注解针对虚谷的批量插入的SQL语法，进行重写
///
///```rust
/// use rbs::value;
/// use rbatis::{Error, RBatis};
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
///  let r = MockTable::insert(rb, &table).await;
///  let r = MockTable::insert_batch(rb, std::slice::from_ref(&table),10).await;
///
///  let tables = MockTable::select_by_map(rb,value!{"id":"1"}).await;
///  let tables = MockTable::select_all(rb).await;
///  let tables = MockTable::select_by_map(rb,value!{"id":["1","2","3"]}).await;
///  let tables = MockTable::select_by_map(rb,value!{"id":"1", "column": ["id", "name"]}).await?;
///
///  let r = MockTable::update_by_map(rb, &table, value!{"id":"1"}).await;
///
///  let r = MockTable::delete_by_map(rb, value!{"id":"1"}).await;
///  //... and more
///  Ok(())
/// }
///
///
/// ```
#[macro_export]
macro_rules! crud {
    ($table:ty{}) => {
        $crate::impl_insert!($table {});
        rbatis::impl_select!($table {});
        rbatis::impl_update!($table {});
        rbatis::impl_delete!($table {});
    };
    ($table:ty{},$table_name:expr) => {
        $crate::impl_insert!($table {}, $table_name);
        rbatis::impl_select!($table {}, $table_name);
        rbatis::impl_update!($table {}, $table_name);
        rbatis::impl_delete!($table {}, $table_name);
    };
}

///PySql: gen sql => INSERT INTO table_name (column1,column2,column3,...) VALUES (value1,value2,value3,...);
///
/// 该注解针对虚谷的批量插入的SQL语法，进行重写
///
/// example:
///```rust
/// use rbatis::{Error, RBatis};
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct MockTable{
///   pub id: Option<String>
/// }
/// rbatis_xugu::impl_insert!(MockTable{});
///
/// //use
/// async fn test_use(rb:&RBatis) -> Result<(),Error>{
///  let table = MockTable{id: Some("1".to_string())};
///  let r = MockTable::insert(rb, &table).await;
///  let r = MockTable::insert_batch(rb, std::slice::from_ref(&table),10).await;
///  Ok(())
/// }
/// ```
///
#[macro_export]
macro_rules! impl_insert {
    ($table:ty{}) => {
        $crate::impl_insert!($table {}, "");
    };
    ($table:ty{},$table_name:expr) => {
        impl $table {
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

            pub async fn insert(
                executor: &dyn rbatis::executor::Executor,
                table: &$table,
            ) -> std::result::Result<rbatis::rbdc::db::ExecResult, rbatis::rbdc::Error> {
                <$table>::insert_batch(executor, std::slice::from_ref(table), 1).await
            }
        }
    };
}

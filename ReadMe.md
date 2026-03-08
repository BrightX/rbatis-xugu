# rbatis-xugu

针对 虚谷数据库 批量插入 SQL 的差异，对 `rbatis::impl_insert!` 和 `rbatis::crud!` 注解进行重写，
使用时请替换为 `rbatis_xugu::impl_insert!` 和 `rbatis_xugu::crud!`。

提供 `XuguTableMapper`，用于 [`table-sync` 插件](https://rbatis.github.io/rbatis.io/#/v4/?id=plugin-table-sync) 。

## 使用

```toml
# Cargo.toml
rbs = "4.6"
rbatis = "4.6"
rbatis-xugu = "4.6"

rbdc-pool-fast = "4.6"
rbdc-xugu = "4.6"
```

`impl_insert!` 和 `crud!` 使用 `rbatis-xugu` 提供的注解。

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MockTable {
    pub id: Option<String>
}
rbatis_xugu::crud!(MockTable{}); //or rbatis_xugu::crud!(MockTable{},"mock_table");

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MockTable2 {
    pub id: Option<String>
}
rbatis_xugu::impl_insert!(MockTable2{}); //or rbatis_xugu::impl_insert!(MockTable2{},"mock_table2");
```


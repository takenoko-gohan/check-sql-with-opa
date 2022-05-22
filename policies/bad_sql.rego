package bad_sql

deny[msg] {
  sql := input[_]
  sql.ast.Delete.selection == null
  msg := sprintf("%s: WHERE句がないDELETE文", [sql.query])
}

deny[msg] {
  sql := input[_]
  sql.ast.Update.selection == null
  msg := sprintf("%s: WHERE句がないUPDATE文", [sql.query])
}

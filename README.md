# check-sql-with-opa

OPAを使用してSQLの判定を行います。

## 利用方法

```sh
# ローカルにOPAのサーバーを立ち上げます
docker compose up -d

# SQLファイルの作成
echo "SELECT * FROM tbl WHERE id = 1;
UPDATE tbl SET price = 100;
DELETE FROM tbl;" > .\test.sql 

# SQLの判定
cargo run -- -f .\test.sql
```

>**Note**
>下記コマンドでヘルプを表示することができます。
>
>```sh
>cargo run -- -h
>```

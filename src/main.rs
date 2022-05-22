use clap::Parser as ClapParser;
use hyper::{body, Body, Client, Method, Request, StatusCode};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser as SqlParser;
use std::fs;

// コマンドの引数やオプション
#[derive(ClapParser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(value_name = "SQL")]
    value: String,

    /// Specify OPA URI
    #[clap(long = "uri", default_value = "http://localhost:8181/v1/data/bad_sql")]
    uri: String,

    /// Check the SQL file
    #[clap(short = 'f', long = "file")]
    is_file: bool,

    /// Show parse results
    #[clap(long = "debug")]
    is_debug: bool,
}

#[derive(Serialize, Debug)]
struct ParseResult {
    query: String,
    ast: Statement,
}

#[derive(Serialize, Debug)]
struct OpaRequest {
    input: Vec<ParseResult>,
}

#[derive(Deserialize, Debug)]
struct OpaResult {
    deny: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct OpaResponse {
    result: OpaResult,
}

// 引数またはファイルから文字列を取得
fn read_contents(value: String, is_file: bool) -> String {
    let contents = if is_file {
        fs::read_to_string(&value).unwrap_or_else(|_| panic!("Unable to read the file {}", &value))
    } else {
        value
    };

    // BOMを無視して返す
    return if contents.chars().next().unwrap() as u64 != 0xfeff {
        contents
    } else {
        let mut chars = contents.chars();
        chars.next();
        String::from(chars.as_str())
    };
}

// SQLをパースした結果を取得
fn parse(contents: String, is_debug: bool) -> Vec<ParseResult> {
    let dialect = GenericDialect {};
    let ast_list = SqlParser::parse_sql(&dialect, contents.as_str()).unwrap();

    if is_debug {
        println!(
            "Parse Result: {}",
            serde_json::to_string_pretty(&ast_list).unwrap()
        );
    }

    // パースするSQLとパース結果をまとめる
    let mut result: Vec<ParseResult> = Vec::new();
    for ast in ast_list.iter() {
        result.push(ParseResult {
            query: ast.to_string(),
            ast: ast.clone(),
        });
    }

    result
}

// OPAのサーバーにリクエストした結果を取得
async fn opa_request(uri: String, input: Vec<ParseResult>) -> OpaResponse {
    let client = Client::new();

    let req_body = serde_json::to_string(&OpaRequest { input }).unwrap();

    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(req_body))
        .unwrap();

    let res = client.request(req).await.unwrap();
    if res.status() != StatusCode::OK {
        panic!("Responses other than HTTP code 200: {}", res.status());
    }

    // レスポンスボディの取得
    let bytes = body::to_bytes(res.into_body()).await.unwrap();
    let res_body = String::from_utf8(bytes.to_vec()).unwrap();

    // レスポンスボディをデシリアライズ
    serde_json::from_str(&res_body).unwrap()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let contents = read_contents(args.value, args.is_file);

    let parse_result = parse(contents, args.is_debug);

    let json = opa_request(args.uri, parse_result).await;

    if json.result.deny.is_empty() {
        println!("There is no problem with SQL");
    } else {
        for deny in json.result.deny.iter() {
            println!("{}", deny);
        }
    }
}

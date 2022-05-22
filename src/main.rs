use clap::Parser as ClapParser;
use hyper::{body, Body, Client, Method, Request, StatusCode};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser as SqlParser;
use std::fs;

#[derive(ClapParser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(value_name = "SQL")]
    value: String,

    /// Specify OPA URI
    #[clap(long = "uri", default_value = "http://localhost:8181/v1/data/bad_sql")]
    uri: String,

    /// Check the SQL file
    #[clap(short, long)]
    file: bool,

    /// Show parse results
    #[clap(long)]
    debug: bool,
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let contents = if args.file {
        fs::read_to_string(args.value.clone())
            .unwrap_or_else(|_| panic!("Unable to read the file {}", args.value))
    } else {
        args.value
    };

    let uri = args.uri;

    let dialect = GenericDialect {};

    let without_bom = if contents.chars().next().unwrap() as u64 != 0xfeff {
        contents.as_str()
    } else {
        let mut chars = contents.chars();
        chars.next();
        chars.as_str()
    };

    let ast_list = SqlParser::parse_sql(&dialect, without_bom).unwrap();
    if args.debug {
        println!(
            "Parse Result: {}",
            serde_json::to_string_pretty(&ast_list).unwrap()
        );
    }

    let mut result: Vec<ParseResult> = Vec::new();
    for ast in ast_list.iter() {
        result.push(ParseResult {
            query: ast.to_string(),
            ast: ast.clone(),
        });
    }

    let client = Client::new();
    let req_body = serde_json::to_string(&OpaRequest { input: result }).unwrap();
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(req_body))
        .unwrap();

    let resp = client.request(req).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("Responses other than HTTP code 200: {}", resp.status());
    }

    let bytes = body::to_bytes(resp.into_body()).await.unwrap();
    let resp_body = String::from_utf8(bytes.to_vec()).unwrap();

    let json: OpaResponse = serde_json::from_str(&resp_body).unwrap();

    if json.result.deny.len() > 0 {
        for deny in json.result.deny.iter() {
            println!("{}", deny);
        }
    } else {
        println!("There is no problem with SQL");
    }
}

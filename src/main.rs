use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::runtime::Builder;
use tokio::task;

#[derive(Deserialize)]
struct ResponseData {
    url: String,
}

async fn make_request(number: u32) -> Result<ResponseData, reqwest::Error> {
    let url = format!(
        "https://tvtcrhau2vo336qa5r66p3bygy0hazyk.lambda-url.us-east-1.on.aws/?cedula=V{}",
        number
    );
    let response = reqwest::get(&url).await?;

    if response.status() == reqwest::StatusCode::BAD_GATEWAY {
        return Ok(ResponseData { url: String::new() });
    }

    let response_data: ResponseData = response.json().await?;
    Ok(response_data)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let num_threads: usize = args
        .get(1)
        .expect("Por favor, proporciona el número de threads como argumento")
        .parse()
        .expect("El argumento debe ser un número entero");

    let runtime = Builder::new_multi_thread()
        .worker_threads(num_threads)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let mut handles = vec![];
        let mut results = HashMap::new();

        for number in (1..=1750110).rev() {
            let handle = task::spawn(async move {
                match make_request(number).await {
                    Ok(response_data) => {
                        println!("Request successful for {}", number);
                        Some((number, response_data.url))
                    }
                    Err(e) => {
                        eprintln!("Request failed for {}: {:?}", number, e);
                        None
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Some((number, url)) = handle.await.unwrap() {
                results.insert(format!("V{}", number), url);
            }
        }

        let json_data = json!(results);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("database.json")
            .expect("No se pudo abrir el archivo");

        file.write_all(json_data.to_string().as_bytes())
            .expect("No se pudo escribir en el archivo");
    });
}

use tokio::fs::OpenOptions;
use string_processing::get_links_from_url;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Mutex;
use std::sync::Arc;
use crate::string_processing;

const ORIGIN_URL: &str = "https://en.wikipedia.org/w/index.php?title=";
const ORIGIN_URL_SUFFIX: &str = "action=raw";

pub(crate) async fn crawler_thread<'a>(guard: &Arc<Mutex<Vec<Vec<String>>>>, resource: (u64, String)) {
    sleep(Duration::from_millis(resource.0)).await;
    println!("info: trying {}", resource.1);
    let _ = write_links(&resource.1).await;

    match get_linked_resources_from_resource(&resource.1).await {
        Ok(e) => {
            // println!("info: adding {:?}", e);
            guard.lock().unwrap().push(e);
        }
        Err(_) => {println!("error: failed on {}", resource.1)},
    }
}

pub(crate) async fn get_linked_resources_from_resource(
    resource: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut f = File::open(format!("data/{}", resource)).await?;
    let mut resources = String::new();
    let _bytes_read = f.read_to_string(&mut resources).await?;

    let resources = resources.lines().into_iter().map(|r| r.to_string());
    return Ok(resources.collect());
}

pub(crate) async fn write_links(resource: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filepath = get_filepath_from_resource(resource);
    if tokio::fs::try_exists(&filepath).await? {
        println!("info: {} already exists", resource);
        return Ok(());
    }

    let mut response = get_links_from_url(&get_url_from_resource(resource)).await?;

    println!("info: trying to create file {}", resource);
    let mut f = match File::create_new(&filepath).await {
        Ok(f) => f,
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => return Ok(()),
            _ => {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&filepath)
                    .await
                {
                    Ok(f) => f,
                    Err(e) => return Err(Box::new(e)),
                }
            }
        },
    };

    response.sort();
    response.dedup();

    for s in response {
        f.write_all(s.as_bytes()).await?;
        f.write_all(b"\n").await?;
    }
    Ok(())
}

pub(crate) fn get_url_from_resource(resource: &str) -> String {
    format!("{ORIGIN_URL}{resource}&{ORIGIN_URL_SUFFIX}")
}

pub(crate) fn get_filepath_from_resource(resource: &str) -> String {
    format!("data/{resource}")
}
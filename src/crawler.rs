use crate::web_data_processing;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use web_data_processing::get_links_from_url;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

const ORIGIN_URL: &str = "https://en.wikipedia.org/w/index.php?title=";
const ORIGIN_URL_SUFFIX: &str = "action=raw";

/*
crawler thread takes a reference to an arc mutex 2d vec of String

Actions:
1) sleep for the random time indicated by resource(ms)
*/
pub(crate) async fn crawler_thread<'a>(
    guard: &Arc<Mutex<Vec<Vec<String>>>>,
    resource: (u64, String),
) {
    sleep(Duration::from_millis(resource.0)).await;
    println!("info: fetching {}", resource.1);
    let _ = write_links(&resource.1).await;

    match get_linked_resources_from_resource(&resource.1).await {
        Ok(e) => {
            // println!("info: adding {:?}", e);
            guard.lock().unwrap().push(e);
        }
        Err(_) => {
            println!("error: failed on {}", resource.1)
        }
    }
}
/*
get_linked_resources_from_resource()
is a function that takes a &str resource

Actions:
1) open file at data/resource
2) read all bytes and return Result<Vec<String>> of lines
*/
pub(crate) async fn get_linked_resources_from_resource(
    resource: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut f = File::open(format!("data/{}", resource)).await?;
    let mut resources = String::new();
    let _bytes_read = f.read_to_string(&mut resources).await?;

    let resources = resources.lines().into_iter().map(|r| r.to_string());
    return Ok(resources.collect());
}

/*
write_links() is a function that takes a &str

Actions:
1) check if file already exists if does, return early
2) call get_links_from_url()
3) write response to a new file with name resource
*/
pub(crate) async fn write_links(resource: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filepath = get_filepath_from_resource("data", resource);
    if tokio::fs::try_exists(&filepath).await? {
        println!("info: {} already exists", resource);
        return Ok(());
    }

    let (returned_resource, mut response) = get_links_from_url(&get_url_from_resource(resource)).await?;

    println!("info: trying to create file {}", returned_resource);
    let mut f = match File::create_new(&filepath).await {
        Ok(f) => f,
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                return Ok(());
            }
            _ => {
                match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&filepath)
                    .await
                {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(Box::new(e));
                    }
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

    println!("info: created {}({:1})", returned_resource, match f.metadata().await {
        Ok(m) => format!("{:.2}KB", m.len() as f32/1024.0),
        Err(_) => "?".to_string(),
    });

    Ok(())
}

pub(crate) fn get_url_from_resource(resource: &str) -> String {
    format!("{ORIGIN_URL}{resource}&{ORIGIN_URL_SUFFIX}")
}

pub(crate) fn get_filepath_from_resource(data_folder_path:&str, resource: &str) -> String {
    format!("{data_folder_path}/{resource}")
}

#[cfg(test)]
mod test_crawler {
    use tokio::fs::{create_dir_all, remove_file, try_exists};
    use super::*;

    #[test]
    fn test_get_url_from_resource() {
        assert_eq!(
            "https://en.wikipedia.org/w/index.php?title=resource&action=raw",
            get_url_from_resource("resource")
        );
    }

    #[test]
    fn test_get_filepath_from_resource() {
        assert_eq!("data/resource", get_filepath_from_resource("data","resource"));
    }

    async fn set_test_state(){
        match create_dir_all("data").await{
            Ok(_) => {}
            Err(_) => {}
        };

        match try_exists("data/JSON").await{
            Ok(true)=>{let _ = remove_file("data/JSON").await;}
            Ok(false)=>{}
            Err(_)=>{assert!(false)}
        }
    }


    async fn reset_test_state(){
        let _ = remove_file("data/JSON").await;        
    }

    #[tokio::test]
    async fn test_write_to_file() {
        set_test_state().await;

        assert!(try_exists("data/JSON").await.is_ok_and(|e|!e));
        assert!(write_links("JSON").await.is_ok());
        assert!(try_exists("data/JSON").await.is_ok_and(|e|e));
        assert!(true);

        reset_test_state().await;
    }


    /*
    this test may break when the JSON wikipedia page breaks, too bad!
     */
    #[tokio::test]
    async fn test_crawler_function(){
        set_test_state().await;
        let vec: Arc<Mutex<Vec<Vec<String>>>> = Arc::new(Mutex::new(Vec::new()));

        crawler_thread(&vec, (0, "JSON".to_string())).await;

        let binding = vec.clone();
        let locked_vec = binding.lock().expect("only one thread was made, should always be able to lock & be returned");
        assert!(locked_vec.len() == 1);
        assert!(locked_vec[0].len() == 160);
        // println!("{:?}", locked_vec[0].len());
        reset_test_state().await;
    }
}

use once_cell::sync::Lazy;
use rand::Rng;
use std::{
    env::args,
    sync::{Arc, Mutex}
};
use tokio::task::JoinSet;

mod string_processing;
mod crawler;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // handle args
    static ARGS: Lazy<Vec<String>> = Lazy::new(|| args().into_iter().collect());
    let origin_resource = match ARGS.get(1) {
        Some(s) => s,
        None => "special:random",
    };

    println!("info: starting from {}", origin_resource);
    crawler::write_links(&origin_resource).await?;

    let resources = crawler::get_linked_resources_from_resource(&origin_resource)
        .await
        .expect("Origin resource");

    let stack_arc_mutex: Arc<Mutex<Vec<Vec<String>>>> = Arc::new(Mutex::new(Vec::from([resources])));

    let mut rng = rand::thread_rng();
    let mut set = JoinSet::new();

    // Main loop to spawn crawler threads
    loop {
        let resources_to_process = {
            let mut vec = stack_arc_mutex.lock().unwrap();
            vec.pop()
        };

        match resources_to_process {
            Some(resource_list) => {
                for resource in resource_list
                    .into_iter()
                    .map(|s| (rng.gen_range(0..100000), s))
                {
                    let stack_arc_mutex_clone = stack_arc_mutex.clone();
                    set.spawn(async move {
                        crawler::crawler_thread(&stack_arc_mutex_clone, resource).await;
                    });
                }
            }
            None => {
                // No more resources to process
                println!("no new resources to process!");
                break;
            }
        }

        // Await completed tasks and potentially add new ones
        while let Some(result) = set.join_next().await  {
            match result {
                Ok(_) => {
                    // Successfully completed a task
                }
                Err(e) => {
                    eprintln!("Task failed: {}", e);
                }
            }
        }

    }
    Ok(())
}




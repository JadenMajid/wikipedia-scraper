use once_cell::sync::Lazy;
use rand::Rng;
use std::{
    env::args,
    io::Error,
    num::ParseIntError,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{create_dir, try_exists},
    task::JoinSet,
};

mod crawler;
mod web_data_processing;

const WARNING_MESSAGE : &str = "warning: You inputted a thread random wait time less than 5s\nif this wait time is too low, you may be IP blocked by wikipedia\ncontinue?[y/n]";

fn get_args() -> Result<(u64, String), Box<dyn std::error::Error>> {
    static ARGS: Lazy<Vec<String>> = Lazy::new(|| args().into_iter().collect());
    let mut random_wait_ms: u64 = 10000;
    let mut origin_resource = "special:random";

    // ARGS[0] = binary path, should skip when parsing args
    match ARGS.len() {
        0 => {}
        1 => {}
        2 => {
            match ARGS[1].parse::<u64>() {
                Ok(i) => {
                    if i < 5000 {
                        let mut line = String::new();
                        println!("{}", WARNING_MESSAGE);
                        std::io::stdin().read_line(&mut line).unwrap();
                        match line.as_str() {
                            "y" => {}
                            _ => return Err(Box::new(std::fmt::Error)),
                        };
                    }
                    random_wait_ms = i
                }
                Err(_) => origin_resource = &ARGS[1],
            };
        }
        _2_plus => match ARGS[1].parse::<u64>() {
            Ok(i) => {
                if i < 5000 {
                    let mut line = String::new();
                    println!("{}", WARNING_MESSAGE);
                    std::io::stdin().read_line(&mut line).unwrap();
                    match line.as_str() {
                        "y" => {}
                        _ => return Err(Box::new(std::fmt::Error)),
                    };
                }
                random_wait_ms = i;
                origin_resource = &ARGS[2]
            }
            Err(_) => {
                origin_resource = &ARGS[1];
                random_wait_ms = match ARGS[2].parse::<u64>() {
                    Ok(i) => {
                        if i < 5000 {
                            let mut line = String::new();
                            println!("{}", WARNING_MESSAGE);
                            std::io::stdin().read_line(&mut line).unwrap();
                            match line.as_str() {
                                "y" => {}
                                _ => return Err(Box::new(std::fmt::Error)),
                            };
                        }
                        i
                    }
                    Err(e) => return Err(Box::new(e)),
                };
            }
        },
    };

    return Ok((random_wait_ms, origin_resource.to_string()));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // handle args
    let random_wait_ms;
    let origin_resource;
    match get_args() {
        Ok(i) => (random_wait_ms, origin_resource) = i,
        Err(e) => {
            println!("exiting program: {}", e);
            return Ok(());
        }
    };

    match try_exists("data/").await {
        Ok(true) => {}
        Ok(false) => {
            match create_dir("data/").await {
                Ok(_) => {}
                Err(e) => {
                    println!("There was an error creating 'data/': {}", e);
                    return Ok(());
                }
            };
        }
        Err(e) => {
            println!("There was an error checking if 'data/' exists: {}", e);
            return Ok(());
        }
    };

    println!("info: starting from {}", origin_resource);
    crawler::write_links(&origin_resource).await?;

    let resources = crawler::get_linked_resources_from_resource(&origin_resource)
        .await
        .expect("Origin resource");

    let stack_arc_mutex: Arc<Mutex<Vec<Vec<String>>>> =
        Arc::new(Mutex::new(Vec::from([resources])));

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
                    .map(|s| (rng.gen_range(0..random_wait_ms), s))
                {
                    let stack_arc_mutex_clone = stack_arc_mutex.clone();
                    println!(
                        "info: spawning thread for {}, waiting {} ms",
                        resource.1, resource.0
                    );
                    set.spawn(async move {
                        crawler::crawler_thread(&stack_arc_mutex_clone, resource).await;
                    });
                }
            }
            None => {
                // No more resources to process
                println!("info: no new resources to process!");
                break;
            }
        }

        // Await completed tasks and potentially add new ones
        while let Some(result) = set.join_next().await {
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

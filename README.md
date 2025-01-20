# WikiScrape

WikiScrape is a web scraper that crawls linked wikipedia resources from pages.

WikiScrape runs off a multithreaded tokio runtime with non blocking threads and execution.

<img src="images/test_run.png"></img>

## Features

- Multithreading
- Async execution

## Structure
<img src = "images/file tree.png"></img>

Code features are modularized by function. 
- web_processing.rs: all of web requests & processing
- crawler.rs: all code required to spawn a single crawler
- main.rs: all code required to spawn multiple crawlers, looping the output of others into the input of new crawlers

The main thread spawns tokio child tasks which check if the query requested resource already exists.
If the resource already exists, the thread returns early. If not, a web request is made and the linked resources are put into a file.

## Notes
- Occasionally, the main thread can appear to hang, this is because the spawned tasks are given a random wait time

## Todo
- [x] page GET
- [x] resource extraction from downloaded pages
- [x] async execution
- [x] multithreading
- [x] unit testing
    - [x] string processing
    - [x] single thread evaluation
- [ ] integration testing
    - inherently non deterministic due to random thread wait times
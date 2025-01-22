# WikiScrape

WikiScrape is a program that scrapes links from wikipedia pages in order to build a graph of its interconnectedness.

WikiScrape runs off a multithreaded tokio runtime with non blocking threads and execution.

<img src="images/test_run.png"></img>

## Fun things

- Multithreading
- Async execution
- Starts from a random wikipedia page when run without a starting page as an arg

## Structure
<img src = "images/file_tree.png"></img>

Code features are modularized by function. 
- web_processing.rs: all of web requests & processing
- crawler.rs: all code required to spawn a single crawler
- main.rs: all code required to spawn multiple crawlers, looping the output of others into the input of new crawlers
- data/: where data is stored(file name is wikipedia resource name with no file suffix)

The main thread spawns tokio child tasks which check if the query requested resource already exists.
If the resource already exists, the thread returns early. If not, a web request is made and the linked resources are put into a file.

## Todo
- [x] page GET
- [x] resource extraction from downloaded pages
- [x] async execution
- [x] multithreading
- [ ] graph generation
- [x] unit testing
    - [x] string processing
    - [x] single thread evaluation
    - [ ] graph generation
- [ ] data collection integration testing
    - inherently non deterministic due to random thread wait times

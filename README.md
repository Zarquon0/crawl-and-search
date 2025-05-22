# Simply Search
Stated simply, this repository is a rust web crawler coupled with a basic search interface for parsing through crawler results. While the search interface provides a nice way of viewing what the crawler has found, but the crawler itself is the heart of this project and works as a stand-alone command line utility. For a more in depth explanation of the project and its features, check out the [writeup](https://github.com/Zarquon0/crawl-and-search/blob/main/Networks%20Final%20Project%20Writeup.pdf).
<div align="center"><img width="690" alt="Screenshot 2025-05-21 at 7 39 39 PM" src="https://github.com/user-attachments/assets/5d3661a5-e6c1-4957-a395-3208b5c0c844"/></div>

## Setup
Clone the repository and
```bash
#cd crawl-and-search/
make setup
```
## Crawler Usage
```bash
#cd crawl-and-search/web-crawler
./crawler --help
```
```bash
Usage: crawler [OPTIONS] [START_POINTS]...

Arguments:
  [START_POINTS]...  List of starting URLs

Options:
  -l, --log-level <LOG_LEVEL>  Set the level of verbosity wanted [default: 1]
      --strict                 Panic on malformed inputs
  -n, --num <NUM>              Number of links to crawl [default: 100]
  -w, --workers <WORKERS>      Number of workers used to crawl [default: 10]
  -d, --db-path <DB_PATH>      Path to database to store results
  -h, --help                   Print help
  -V, --version                Print version
```
Example usage (this example uses 5 worker threads to crawl the first 1000 links encountered starting with the two supplied urls):
```bash
./crawler -n 1000 -w 5 http://google.com https://github.com/Zarquon0/crawl-and-search
```
One note: That `<DB_PATH>` should be the path to a properly set up SQLite database file (the crawler assumes a certain form). After running `make setup`, there should be a `search_db.db` file in `search_engine_app/` that is set up for that purpose exactly, so I'd use that one. 
## Web App Usage
```bash
#cd crawl-and-search/
make run_deploy
```
Check out http://127.0.0.1:5000 and you should see the app running



mod prelude;
mod crawler_datatypes;
mod crawler_utilities;
mod database_interaction;
mod url_tree;

use crate::prelude::*;
use crate::crawler_datatypes::*;
use crate::crawler_utilities::*;
use crate::database_interaction::*;
use clap::Parser;
use reqwest::{blocking, StatusCode};

//TODO:
//Fix scary deadlocking bug - DONE
//Check if a url is a bad one that's already been searched - DONE
//Add async for the web requesting
//Random URL picking - DONE
//Add sqlite database updating funcationality - DONE
//Actually add this stuff to the github - DONE
//Load database in at the beginning
//Make extensibility feature

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set the level of verbosity wanted
    #[arg(short, long, default_value_t=1)]
    log_level: u8,
    /// Panic on malformed inputs
    #[arg(long)]
    strict: bool,
    /// Number of links to crawl
    #[arg(short, long, default_value_t=100)]
    num: u32,
    /// Number of workers used to crawl
    #[arg(short, long, default_value_t=10)]
    workers: u8,
    /// Path to database to store results
    #[arg(short, long)]
    db_path: Option<PathBuf>,
    /// List of starting URLs
    #[arg()]
    start_points: Vec<String>
}

fn main() {
    //Parse arguments
    let args = Args::parse();
    let mut start_points = Vec::new();
    for url in args.start_points {
        match cleanse_url(&url) {
            Some(clean_url) => start_points.push(clean_url),
            None => if args.strict { panic!("Input URL {url} not well formed") }
        }
    }
    if start_points.is_empty() { return eprintln!("No valid starting URLs supplied; exiting :("); }
    //Create all important objects
    let pbar = Arc::new(if args.log_level > 0 { ProgressBar::new(args.num as u64) }
        else { ProgressBar::hidden() }); 
    pbar.set_style(ProgressStyle::default_bar().template("[{bar:40.green/red}] {pos}/{len} {eta} {msg}").unwrap().progress_chars("|>-"));
    let options = DispOptions::new(args.log_level, pbar.clone());
    let disp = make_disp(options.clone());
    let site_map = Arc::new(SiteMap::new(args.num, pbar.clone()));
    let public_links = Arc::new(LinkList::new(start_points, args.workers));
    //Load database if need be
    if let Some(db_path) = &args.db_path {
        match load_db(db_path, site_map.clone()) {
            Ok(()) => {},
            Err(e) => return eprintln!("DATABASE ERROR: {e}\nTLDR; Couldn't read properly from database - it's either misconfigured or the path is incorrect")
        }
    }
    //Spawn crawlers
    let timer = Instant::now();
    disp(format!("Let the crabby crawling begin!"), 1);
    let mut crawly_bois = Vec::new();
    for worker_id in 0..args.workers {
        let site_map_clone = site_map.clone();
        let pub_links_clone = public_links.clone();
        let options_clone = options.clone();
        let handle = thread::spawn(move || get_crawlin(worker_id, site_map_clone, pub_links_clone, options_clone));
        crawly_bois.push(handle);
    }
    //Wait for crawlers to terminate
    let mut outstanding = 0;
    let mut tot_request_time = Duration::new(0,0);
    let mut tot_work_time = Duration::new(0,0);
    for crawly_boi in crawly_bois {
        let wdata = crawly_boi.join().expect("Crawly Boi panicked :(");
        outstanding += wdata.outstanding;
        tot_request_time += wdata.req_time;
        tot_work_time += wdata.tot_time;
    }
    let elapsed = timer.elapsed();
    pbar.finish();
    disp(format!("Finished crawling!\nSites crawled: {}\nOutstanding links: {}\nTime Crawling: {:?}\nRequest Time: {:?}\nWork Time: {:?}", site_map.len(), outstanding, elapsed, tot_request_time, tot_work_time), 1);
    //Add results to database, if specified
    match &args.db_path {
        Some(path) => update_db(path, site_map),
        None => {} //Do nothing, we're done here 
    }
}

//fn parse_args(args: Vec<String>) -> Vec<String> {}

fn get_crawlin(worker_id: u8, site_map: Arc<SiteMap>, pub_links: Arc<LinkList>, options: DispOptions) -> WorkerData {
    //println!("Initiated!");
    let start = Instant::now();
    let disp = make_disp(options);
    let mut our_links = LocalUrls::new();
    let client = blocking::Client::builder().timeout(Duration::from_secs(3)).build().unwrap();
    let mut request_time = Duration::new(0, 0);
    //Debug Timers
    //let mut parsing = Duration::new(0,0);
    //let mut grabbing = Duration::new(0,0);
    let mut url_fetching = Duration::new(0,0);
    let mut url_checking = Duration::new(0,0);
    //let mut crawling = Duration::new(0,0);
    loop {
        //Grab next URL, from local list if possible and public link list if not
        let url_fetch: Instant = Instant::now();
        let next_url = match our_links.next() {
            Some(clean_url) => clean_url,
            None => pub_links.next()
        };
        url_fetching += url_fetch.elapsed();
        //Check if URL has already been crawled and skip if so
        let url_check = Instant::now();
        let already_crawled = site_map.contains_key(&next_url);
        url_checking += url_check.elapsed();
        if already_crawled { continue }
        //Crawl page and update relevant objects
        let crawl_time = Instant::now();
        let crawl_results = crawl(&client, &next_url);
        request_time += crawl_time.elapsed();
        match crawl_results {
            Ok(mut parsed) => {
                disp(format!("INSERTING: {}", next_url), 3);
                if site_map.insert(next_url, parsed.data) { break; };
                if !parsed.links.is_empty() {
                    let num_pub_add = cmp::min(pub_links.should_add(), (parsed.links.len() - 1)as u8);
                    if num_pub_add > 0 { pub_links.add(parsed.links.drain(..num_pub_add as usize).collect())}
                }
                //Add links we found to local links to crawl
                our_links.extend(parsed.links);
            },
            Err(e) => {
                disp(format!("CRAWL ERROR: {e}"), 3);
                site_map.insert_bad(next_url);
            }
        };
    }
    disp(format!("---CRAB {worker_id}---"), 2);
    disp(format!("Crawling: {:?}\nUrl fetching: {:?}\nUrl checking: {:?}", request_time, url_fetching, url_checking), 2);
    WorkerData::new(our_links.len(), request_time, start.elapsed())
}

fn crawl(client: &blocking::Client, cleansed_url: &String) -> Result<ParsedPage> {
    let url = dirty_url(&cleansed_url);
    let response = client.get(&url).send()?;
    match response.status() {
        StatusCode::OK => {
            let page = response.text()?;
            let parsed = parse_page(page);
            Ok(parsed)
        },
        code => Err(Error::msg(format!("Bad Status Code: {code:?}")))
    }
}
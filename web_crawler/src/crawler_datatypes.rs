use crate::prelude::*;
use rand::{Rng, rngs::ThreadRng};

const SKIP_AMOUNT: usize = 5;
const SHAKE_THRESH: u8 = 10;

pub struct SiteMap {
    map: RwLock<HashMap<String, PageData>>,
    len: RwLock<usize>,
    capacity: usize,
    bad_pages: RwLock<HashSet<String>>,
    previously_searched: RwLock<HashSet<String>>,
    pbar: Arc<ProgressBar>
}
impl SiteMap {
    pub fn new(capacity: u32, pbar: Arc<ProgressBar>) -> SiteMap {
        SiteMap { map: RwLock::new(HashMap::<String, PageData>::new()), len: RwLock::new(0), capacity: capacity as usize, bad_pages: RwLock::new(HashSet::<String>::new()), previously_searched: RwLock::new(HashSet::new()), pbar }
    }
    pub fn insert(&self, key: String, data: PageData) -> bool {
        let mut len = self.len.write();
        if *len == self.capacity { return true; } //Can't insert, we done
        //println!("Inserting: {}", key);
        let mut map = self.map.write();
        match map.insert(key, data) {
            Some(_) => false, //Value was already in the map, do nothing
            None => {
                //Increment length and progress bar
                self.pbar.inc(1);
                *len += 1;
                //If at capacity, we're done, so return true
                *len == self.capacity
            }
        }
    }
    pub fn insert_bad(&self, bad_url: String) {
        let mut bad_pages = self.bad_pages.write();
        bad_pages.insert(bad_url);
    }
    pub fn insert_previously(&self, prev_url: String) {
        let mut previously_searched = self.previously_searched.write();
        previously_searched.insert(prev_url);
    }
    pub fn contains_key(&self, key: &String) -> bool { 
        let map = self.map.read();
        let bad_pages = self.bad_pages.read();
        let previously_searched = self.previously_searched.read();
        map.contains_key(key) || bad_pages.contains(key) || previously_searched.contains(key)
    }
    pub fn len(&self) -> usize { 
        let len = self.len.read();
        *len
    }
    pub fn get_map(&self) -> RwLockReadGuard<HashMap<String, PageData>> {
        self.map.read()
    }
}


pub struct LinkList {
    links: parking_lot::RwLock<VecDeque<String>>,
    alert: RwCondvar,
    capacity: u8
}
impl LinkList {
    pub fn new(starting_points: Vec<String>, capacity: u8) -> LinkList {
        LinkList { links: parking_lot::RwLock::new(VecDeque::from(starting_points)), alert: RwCondvar::new(), capacity }
    }
    pub fn next(&self) -> String {
        let mut links = self.links.write();
        while links.is_empty() { //Be VERY careful here - waitw contains deadlocking risk
            self.alert.waitw(&mut links);
        }
        let next_url = links.pop_front().unwrap();
        if !links.is_empty() { self.alert.notify_one(); }
        next_url
    }
    pub fn add(&self, new_links: Vec<String>) {
        if !new_links.is_empty() {
            let mut links = self.links.write();
            if links.len() + new_links.len() > self.capacity as usize { eprintln!("Adding beyond capacity, shouldn't do this") } //Checks for intended behavior 
            links.extend(new_links);
            self.alert.notify_one();
        }
    }
    pub fn should_add(&self) -> u8 {
        let links = self.links.read();
        self.capacity - links.len() as u8
    }
}

pub struct LocalUrls {
    urls: Vec<String>, 
    last_domain: Option<String>,
    iter_cnt: u8,
    idx: usize,
    rand_rng: ThreadRng
}
impl LocalUrls {
    pub fn new() -> LocalUrls {
        LocalUrls { urls: Vec::new(), last_domain: None, iter_cnt: 0, idx: 0, rand_rng: rand::thread_rng() }
    }
    pub fn next(&mut self) -> Option<String> {
        if self.urls.is_empty() { return None }
        //Let's shake things up! Completely change the current location in the local list of links periodically
        self.iter_cnt += 1;
        if self.iter_cnt == SHAKE_THRESH {
            self.idx = self.rand_rng.gen_range(0..self.urls.len());
            self.iter_cnt = 0;
        }
        //If the next url to search has the same domain as the previous one, skip ahead a little
        let next_url = &self.urls[self.idx];
        match self.last_domain.clone() {
            Some(last_domain) => {
                if LocalUrls::domain(next_url) == last_domain {
                    self.incr(SKIP_AMOUNT);
                    //next_url = &self.urls[self.idx];
                }
            }
            None => {}
        }
        //Remove and return url
        let next_url = self.urls.remove(self.idx);
        //self.incr(1); No need to increment - we removed one
        self.incr(0);
        Some(next_url)
    }
    fn incr(&mut self, num: usize) {
        self.idx += num;
        while self.idx >= self.urls.len() && !self.urls.is_empty() {
            self.idx -= self.urls.len()
        }
    }
    fn domain(url: &String) -> String {
        let domain_match = Regex::new(r"^.+(/|$)").unwrap();
        match domain_match.find(&url) {
            Some(mat) => mat.as_str().to_string(),
            None => panic!("No domain in input string - this should never happen")
        }
    }
    pub fn extend(&mut self, urls: Vec<String>) {
        self.urls.extend(urls)
    }
    pub fn len(&self) -> usize {
        self.urls.len()
    }
}

//CREDIT: Amanieu in https://github.com/Amanieu/parking_lot/issues/165 for the following - I have no idea why std::Convar doesn't support RwLocks:
struct RwCondvar {
    cv: Condvar,
    mtx: Mutex<()>
}
impl RwCondvar {
    pub fn new() -> RwCondvar {
        RwCondvar { cv: Condvar::new(), mtx: Mutex::new(()) }
    }
    pub fn waitw<T>(&self, gaurd: &mut parking_lot::RwLockWriteGuard<'_, T>) {
        let mut locked = self.mtx.lock();
        parking_lot::RwLockWriteGuard::unlocked(gaurd, || {
            self.cv.wait(&mut locked);
        })
    }
    pub fn notify_one(&self) -> bool { self.cv.notify_one() }
    //pub fn notify_all(&self) -> usize { self.cv.notify_all() }
}

pub struct ParsedPage {
    pub data: PageData,
    pub links: Vec<String> //List of cleansed urls
}

pub struct PageData {
    pub title: Option<String>
    //Could be expanded...
}

#[derive(Clone)]
pub struct DispOptions {
    pub log_level: u8,
    pub pbar: Arc<ProgressBar>
}
impl DispOptions {
    pub fn new(log_level: u8, pbar: Arc<ProgressBar>) -> DispOptions {
        DispOptions { log_level, pbar }
    }
}

pub struct WorkerData {
    pub outstanding: usize,
    pub req_time: Duration, 
    pub tot_time: Duration, 
}
impl WorkerData {
    pub fn new(outstanding: usize, req_time: Duration, tot_time: Duration) -> WorkerData {
        WorkerData { outstanding, req_time, tot_time }
    }
}

mod tests {
    use super::*;
    //LocalURls Tests
    #[test]
    fn next_simple() {
        let mut lu = LocalUrls::new();
        let urls = ["gwango.lol/hello", "google.com/"].to_vec().iter().map(|url| url.to_string()).collect();
        lu.extend(urls);
        assert_eq!(lu.next(), Some("gwango.lol/hello".to_string()));
        assert_eq!(lu.next(), Some("google.com/".to_string()));
    }
    #[test]
    fn next_complex() {
        let mut lu = LocalUrls::new();
        let urls: Vec<String> = ["gwango.lol/hello", "gwango.lol/", "google.com/", "yahoo.com/stuff", "other_site/thingy"].to_vec().iter().map(|url| url.to_string()).collect();
        lu.extend(urls.clone());
        assert_eq!(lu.next(), Some("gwango.lol/hello".to_string()));
        let mut idx = 1 + SKIP_AMOUNT;
        while idx >= urls.len() {
            idx -= urls.len();
        }
        assert_eq!(lu.next(), Some(urls[idx].to_string()));
    }
}
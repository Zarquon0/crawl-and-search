use crate::prelude::*;
use crate::crawler_datatypes::*;
use crate::url_tree::valid_url_char;

pub fn parse_page(page: String) -> ParsedPage {
    ParsedPage {
        data: PageData { title: find_title(&page) },
        links: find_links(&page)
    }
}

///Finds the titls of a webpage, returning None if it cannot find a title
pub fn find_title(page: &String) -> Option<String> {
    let title_match = Regex::new(r"<title\s*.*?>.+?</title>").unwrap();
    match title_match.find(page) {
        Some(title) => {
            let extract_match = Regex::new(r">.+?<").unwrap();
            let innards = extract_match.find(title.as_str()).unwrap().as_str();
            let innards = &innards[1..innards.len()-1];
            Some(innards.to_string())
        }
        None => None
    }
}

///Finds all links contained within a webpage, cleans them, and returns a vector of them
pub fn find_links(page: &String) -> Vec<String> {
    let link_match = Regex::new(r#"<a.+?href=("|').+?("|').*?>"#).unwrap();
    let mut urls = Vec::new();
    link_match.find_iter(page).for_each(|mat| {
        let extract_match = Regex::new(r#"href=("|').+?("|')"#).unwrap();
        let innards = extract_match.find(mat.as_str()).unwrap().as_str();
        let url_str = &innards[6..innards.len()-1]; //Eliminates href=" and the closing " from the string
        if let Some(cleansed) = cleanse_url(url_str) {
            urls.push(cleansed)
        }
    });
    urls
}

///Cleanses an input url by removing http://, any trailing queries (?thing=sfd...), and whitespace
///Returns None if not a valid url
pub fn cleanse_url(url: &str) -> Option<String> {
    let proto_match  = Regex::new(r"https?://").unwrap();
    let query_match = Regex::new(r"\?.*").unwrap();
    if proto_match.is_match(url) {
        let cut_proto = proto_match.replace(url, "");
        let cut_query = query_match.replace(&cut_proto, "");
        let trimmed = cut_query.trim().to_string();
        if valid_url(&trimmed) { Some(trimmed) }
        else { None }
    } else { None }
}

///Turns a cleansed url into a usable url
pub fn dirty_url(cleansed_url: &String) -> String { format!("https://{cleansed_url}") }

///Ensures a cleansed url contains only valid characters
pub fn valid_url(url: &String) -> bool {
    for ch in url.chars() {
        if !valid_url_char(ch) { return false; }
    }
    true
}

///Makeshift logger
pub fn make_disp(opts: DispOptions) -> impl Fn(String, u8) {
    move |msg: String, msg_level: u8| {
        if opts.log_level >= msg_level { 
            if opts.pbar.is_hidden() || opts.pbar.is_finished() { println!("{msg}") }
            else { opts.pbar.println(msg); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //find_title Tests
    #[test]
    fn find_title_simple() {
        let page = "stuff stuff stuff more stuff <title>Title</title> and more stuff out here too".to_string();
        assert_eq!(find_title(&page), Some(String::from("Title")));
    }
    #[test]
    fn find_title_complex() {
        let page = "stuff stuff <a href=\"gwango.lol\">yeah</a> more stuff <title attr></tItle> </title> and more stuff out here too".to_string();
        assert_eq!(find_title(&page), Some(String::from("</tItle> ")));
    }
    #[test]
    fn find_title_no_title() {
        let page = "stuff stuff more stuff <title>Title<title> and more stuff out here too".to_string();
        assert_eq!(find_title(&page), None);
    }
    //find_links Tests
    #[test]
    fn find_links_simple() {
        let page = "stuff stuff stuff more stuff <a href=\"https://heybudy\">Title</a> and more stuff out here too".to_string();
        assert_eq!(find_links(&page), vec!["heybudy".to_string()]);
    }
    #[test]
    fn find_links_complex() {
        let page = "stuff stuff <a attr href=\"gwango.lol\">yeah</a> more stuff <a href='http://wassup.com/its_ya_boy?actually=dont_include_this' attr></tItle <a href='https://please I good url'> </title> and more href=\"  https://weee  \" out here too".to_string();
        assert_eq!(find_links(&page), vec!["wassup.com/its_ya_boy".to_string()]);
    }
    #[test]
    fn find_links_no_links() {
        let page = "stuff stuff more stuff <title>Title<title> and more stuff out here too".to_string();
        assert_eq!(find_links(&page), Vec::<String>::new());
    }
    //cleanse_url Tests
    #[test]
    fn cleanse_url_simple() {
        let url = "https://gwango.lol".to_string();
        assert_eq!(cleanse_url(&url), Some("gwango.lol".to_string()));
    }
    #[test]
    fn cleanse_url_complex() {
        let url = "\n http://cs.gwango.lol/sub-dir/anutha_/page.html?key=val  ".to_string();
        assert_eq!(cleanse_url(&url), Some("cs.gwango.lol/sub-dir/anutha_/page.html".to_string()));
    }
    #[test]
    fn cleanse_url_bad_link() {
        let url = "http:/gwango.lol".to_string();
        assert_eq!(cleanse_url(&url), None);
    }
    #[test]
    fn cleanse_url_bad_chars() {
        let url = "https://gwango lol".to_string();
        assert_eq!(cleanse_url(&url), None);
    }
}
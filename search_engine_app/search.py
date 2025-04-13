from search_backend import app, db
from models import Site
import re

# CREDIT: ChatGPT 4o 
common_words = [
    # Articles
    "the", "a", "an",

    # Conjunctions
    "and", "but", "or", "nor", "so", "for", "yet", "although", "because", "since", "unless", 
    "while", "whereas", "though", "after", "before", "once", "if", "as", "whether",

    # Prepositions
    "in", "on", "at", "by", "with", "about", "against", "between", "into", "through", "during", 
    "before", "after", "above", "below", "to", "from", "up", "down", "over", "under", "of", 
    "off", "out", "around", "near", "along", "throughout", "until", "within", "without",

    # Pronouns
    "he", "she", "it", "we", "they", "you", "I", "me", "him", "her", "us", "them", 
    "my", "your", "his", "its", "our", "their", "mine", "yours", "hers", "ours", "theirs", 
    "this", "that", "these", "those", "who", "whom", "whose", "which", "what", "where", 
    "when", "why", "how", "anyone", "someone", "everyone", "no one", "none", "nothing",

    # Other function words
    "is", "are", "was", "were", "be", "been", "being", "am", "do", "does", "did", 
    "can", "could", "will", "would", "shall", "should", "may", "might", "must", "have", "has", 
    "had", "not", "no", "yes", "all", "any", "some", "few", "more", "most", "much", "many", 
    "each", "every", "either", "neither", "both", "only", "just", "even", "also", "always", 
    "never", "again", "perhaps"
]

def execute_search(raw_query):
    # Clean up terms if possible
    terms = raw_query.split(' ')
    clean_terms = []
    for term in terms:
        if not term in common_words:
            clean_terms.append(term)
    if len(clean_terms) == 0: 
        clean_terms = terms
    print("Clean terms:",clean_terms)
    # Get results for each term and label with frequency
    res_dict = {}
    total_title_map = {}
    for term in clean_terms:
        (urls, title_map) = search_term(term)
        for url in urls:
            if url in res_dict:
                res_dict[url] += 1
            else:
                res_dict[url] = 1
        total_title_map = total_title_map | title_map
    # Organize results into single list, dirty, package in title, and return
    sorted_results = sorted(res_dict, key=lambda ky: res_dict[ky], reverse=True)
    final_results = map(lambda url: {"url": url, "title": total_title_map[url]}, sorted_results)
    return final_results

#def find_domain(url):
#    dom_pattern = r"^.+/"
#    domain = re.search(dom_pattern, url)
#    return domain.group()

def search_term(term):
    term = term.lower().strip()
    delimeter_pattern = r"[%/\.-]"
    site_map = None
    with app.app_context():
        site_map = Site.query.all()
    urls = []
    title_map = {}
    for entry in site_map:
        match entry.title:
            case None: pass #Do nothing
            case title:
                if term in title.lower().split(' '):
                    urls.append(entry.url)
                    title_map[entry.url] = entry.title
                    continue
        if term in re.split(delimeter_pattern, entry.url):
            urls.append(entry.url)
            title_map[entry.url] = entry.title
    return (urls, title_map)

# def dirty_url(url):
#    return "https://"+url
from search_backend import app
from flask import render_template
from search_form import SearchForm
from backend_utils import log
from search import execute_search
import subprocess

starting_url = "theuselessweb.com/"

# Helpers
def search_more(urls, num=100):
    urls = list(map(lambda url: "https://"+url, urls))
    #all_urls = ' '.join(urls)
    command_list = ["./crawler", "-l", "0", "-n", f"{num}", "-d", "./search_db.db"]+urls
    print(command_list)
    subprocess.run(command_list)

# Routes
@app.route('/', methods=['GET', 'POST'])
def index():
    searchform = SearchForm()
    sites = None
    if searchform.validate_on_submit():
        sites = execute_search(searchform.term.data)
    else:
        log("Failed to validate search form...")
    return render_template("search_engine.html", sites=sites, sform=searchform)
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from os import path
from pathlib import Path
from backend_utils import log

# Set up app
app = Flask(__name__)

# Configue app
app.config["SECRET_KEY"] = "Du(kthu1u'sMin1on5"
app.config["SQLALCHEMY_DATABASE_URI"] = f'sqlite:///{path.abspath("search_db.db")}'
app.config["SQLALCHEMY_TRACK_MODIFICATIONS"] = False

# Init key objects
db = SQLAlchemy(app)

# Bring models into scope
from models import *

# Set up database if nonexistent
if not Path("search_db.db").exists():
    with app.app_context():
        db.create_all()
    log("No database currently - created new one")

# Run views
import views
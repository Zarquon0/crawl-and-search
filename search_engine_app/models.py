from search_backend import db

# Models
class Site(db.Model):
    url = db.Column(db.String(100), primary_key=True)
    title = db.Column(db.String(50))
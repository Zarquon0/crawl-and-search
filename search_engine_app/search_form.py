from flask_wtf import FlaskForm
from wtforms import StringField, SubmitField

class SearchForm(FlaskForm):
    term = StringField("Search Term", render_kw={'placeholder': 'What Would You Like to See?'})
    submit = SubmitField("Search")
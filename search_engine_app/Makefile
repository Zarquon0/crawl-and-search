ROOT_DIR := $(shell pwd)

.PHONY: setup, run_dev, run_deploy, clean

setup:
	. bin/activate; \
	pip3 install flask sqlalchemy Flask-SQLAlchemy Flask-WTF gunicorn
	export FLASK_APP="search_backend.py"

run_dev:
	. bin/activate; \
	flask run

run_deploy:
	. bin/activate; \
	gunicorn -w 3 -b 127.0.0.1:5000 wsgi:app

clean:
	rm search_db.db
	rm log.txt
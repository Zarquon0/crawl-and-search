.PHONY: setup, run_dev, run_deploy, clean

crawler:
	$(MAKE) -C web_crawler crawler
	cp web_crawler/crawler search_engine_app/

setup: crawler
	mv search_engine_app old_stuff
	python3 -m venv search_engine_app
	cp -r old_stuff/* search_engine_app/
	rm -rf old_stuff
	$(MAKE) -C search_engine_app setup

default_db: crawler
	cd search_engine_app && ./crawler -d ./search_db.db https://allmyfaves.com/

run_dev:
	$(MAKE) -C search_engine_app run_dev

run_deploy:
	$(MAKE) -C search_engine_app run_deploy
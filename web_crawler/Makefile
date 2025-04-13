ROOT_DIR := $(shell pwd)

.PHONY: crawler

crawler:
	cargo build --manifest-path $(ROOT_DIR)/Cargo.toml
	cp $(ROOT_DIR)/target/debug/web_crawler $(ROOT_DIR)/crawler
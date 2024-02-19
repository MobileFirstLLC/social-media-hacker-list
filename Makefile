SHELL := /bin/bash

all:
	cd checker && cargo build --release
	cp checker/target/release/checker check
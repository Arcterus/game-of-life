ifneq ($(CFG),)
	RUST_CFG := --cfg $(CFG)
endif

all: life

life: src/life.rs piston
	rustc -O $(RUST_CFG) -L src/piston-workspace/piston-symlinks/ -o $@ $<

piston:
	git submodule update --init --recursive && cd src/piston-workspace && chmod +x build.sh && ./build.sh && make
	touch $@

clean-conway:
	rm -f life

clean: clean-conway
	rm -f piston
	cd src/piston-workspace && make clean

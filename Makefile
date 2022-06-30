kernel.elf: mikanos_kernel_rust/src/
	cd mikanos_kernel_rust && cargo build --release && cp ../target/x86_64-unknown-none-mikankernel/release/mikanos_kernel_rust ../kernel.elf

.PHONY: all
all: kernel.elf

.PHONY: clean
clean:
	rm -fr kernel.elf disk.img

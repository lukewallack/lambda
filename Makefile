SRC=$(wildcard *.rs)

all: main

main: $(SRC)
	rustc main.rs -o main

.PHONY: test
test:
	rustc --test main.rs -o main-tests
	-./main-tests
	@rm -f main-tests


.PHONY: clean
clean:
	rm -f main main-tests

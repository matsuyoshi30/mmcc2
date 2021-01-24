mmcc2:
	cargo build

test:
	./test.sh

clean:
	rm -f mmcc2 *.o *~ tmp*

.PHONY: test clean

#!/bin/bash
mmcc2="./target/debug/mmcc2"

assert() {
    expected="$1"
    input="$2"

    ${mmcc2} "$input" > tmp.s
    gcc -o tmp tmp.s
    ./tmp
    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert 0 0
assert 42 42
assert 6 3+3
assert 3 4-1
assert 4 5+1-2
assert 5 4-2+3
assert 10 ' 12 - 2 '
assert 12 '10+8/4'
assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
assert 10 '-10+20'
assert 15 '-3*-5'
assert 20 '- - +20'

echo OK

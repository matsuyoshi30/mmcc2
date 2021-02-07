#!/bin/bash
mmcc2="./target/debug/mmcc2"

cat <<EOF | cc -x c -c -o tmp2.o -
int testFunc1() { return 5; }
int testFunc2(int x, int y) { return x+y; }
int testFunc3(int a, int b, int c, int d, int e, int f) { return a+b+c+d+e+f;}
EOF

assert() {
    expected="$1"
    input="$2"

    ${mmcc2} "$input" > tmp.s
    gcc -fPIC -o tmp tmp.s tmp2.o
    ./tmp
    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert 0 'main() { return 0; }'
assert 42 'main() { return 42; }'
assert 6 'main() { return 3+3; }'
assert 3 'main() { return 4-1; }'
assert 4 'main() { return 5+1-2; }'
assert 5 'main() { return 4-2+3; }'
assert 10 'main() { return 12 - 2 ; }'
assert 12 'main() { return 10+8/4; }'
assert 47 'main() { return 5+6*7; }'
assert 15 'main() { return 5*(9-6); }'
assert 4 'main() { return (3+5)/2; }'
assert 10 'main() { return -10+20; }'
assert 15 'main() { return -3*-5; }'
assert 20 'main() { return - - +20; }'
assert 1 'main() { return 5>3; }'
assert 0 'main() { return 5>5; }'
assert 1 'main() { return 3<5; }'
assert 0 'main() { return 3<3; }'
assert 0 'main() { return 6>=9; }'
assert 1 'main() { return 9>=6; }'
assert 0 'main() { return 9<=6; }'
assert 1 'main() { return 6<=9; }'
assert 0 'main() { return 2==3; }'
assert 1 'main() { return 3==3; }'
assert 1 'main() { return 2!=3; }'
assert 0 'main() { return 3!=3; }'
assert 5 'main() { return a=5; }'
assert 2 'main() { return b=3-1; }'
assert 1 'main() { return c=5>3; }'
assert 10 'main() { return foo=10; }'
assert 22 'main() { return _val1=22; }'
assert 123 'main() { bar=10; return bar=123; }'
assert 22 'main() { a=13; return b=a+9; }'
assert 30 'main() { foo=30; return foo; }'
assert 45 'main() { a=15; b=3; return a*b; }'
assert 10 'main() { if (5>3) return 10; }'
assert 20 'main() { if (5<3) return 10; else return 20; }'
assert 30 'main() { foo=5; if (5<3) return 10; else if (foo==4) return 20; else return 30; }'
assert 40 'main() { foo=5; if (5>3) if (foo==5) return 40; else return 20; else return 30; }'
assert 5 'main() { i=1; while (i<5) i=i+1; return i; }'
assert 10 'main() { x=0; for (i=0; i<10; i=i+1) x=x+1; return x; }'
assert 3 'main() { for (;;) return 3; return 5; }'
assert 55 'main() { i=0; j=0; for (i=0; i<=10; i=i+1) j=i+j; return j; }'
assert 100 'main() { a=0; for (i=0; i<10; i=i+1) if (i==5) a=100; return a; }'
assert 6 'main() { if (5>3) { a=3; b=2; c=a*b; } return c; }'
assert 100 'main() { ret=0; for (i=0; i<10; i=i+1) { j=0; while (j<10) { ret=ret+1; j=j+1; } } return ret; }'
assert 5 'main() { return testFunc1(); }'
assert 3 'main() { return testFunc2(1, 2); }'
assert 21 'main() { return testFunc3(1, 2, 3, 4, 5, 6); }'

echo OK

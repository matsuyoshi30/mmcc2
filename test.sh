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

assert 0 '0;'
assert 42 '42;'
assert 6 '3+3;'
assert 3 '4-1;'
assert 4 '5+1-2;'
assert 5 '4-2+3;'
assert 10 ' 12 - 2 ;'
assert 12 '10+8/4;'
assert 47 '5+6*7;'
assert 15 '5*(9-6);'
assert 4 '(3+5)/2;'
assert 10 '-10+20;'
assert 15 '-3*-5;'
assert 20 '- - +20;'
assert 1 '5>3;'
assert 0 '5>5;'
assert 1 '3<5;'
assert 0 '3<3;'
assert 0 '6>=9;'
assert 1 '9>=6;'
assert 0 '9<=6;'
assert 1 '6<=9;'
assert 0 '2==3;'
assert 1 '3==3;'
assert 1 '2!=3;'
assert 0 '3!=3;'
assert 5 'a=5;'
assert 2 'b=3-1;'
assert 1 'c=5>3;'
assert 10 'foo=10;'
assert 22 '_val1=22;'
assert 123 'bar=10; bar=123;'
assert 22 'a=13; b=a+9;'
assert 5 'return 5;'
assert 30 'foo=30; return foo;'
assert 45 'a=15; b=3; return a*b;'
assert 10 'if (5>3) return 10;'
assert 20 'if (5<3) return 10; else return 20;'
assert 30 'foo=5; if (5<3) return 10; else if (foo==4) return 20; else return 30;'
assert 40 'foo=5; if (5>3) if (foo==5) return 40; else return 20; else return 30;'
assert 5 'i=1; while (i<5) i=i+1; return i;'
assert 10 'x=0; for (i=0; i<10; i=i+1) x=x+1; return x;'
assert 3 'for (;;) return 3; return 5;'
assert 55 'i=0; j=0; for (i=0; i<=10; i=i+1) j=i+j; return j;'
assert 100 'a=0; for (i=0; i<10; i=i+1) if (i==5) a=100; return a;'
assert 6 'if (5>3) { a=3; b=2; c=a*b; } return c;'
assert 100 'ret=0; for (i=0; i<10; i=i+1) { j=0; while (j<10) { ret=ret+1; j=j+1; } } return ret;'
assert 5 '{ return testFunc1(); }'
assert 3 '{ return testFunc2(1, 2); }'
assert 21 '{ return testFunc3(1, 2, 3, 4, 5, 6); }'

echo OK

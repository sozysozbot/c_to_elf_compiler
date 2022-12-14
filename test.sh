#!/bin/bash

cd $(dirname $0)
cargo build

check_inner() {
  TMPDIR=$(mktemp -d --tmpdir=testwork)
  expected="$1"
  input="$2"
  stdout_expected="$3"

  (cd $TMPDIR && ../../target/debug/c_to_elf_compiler "$input")
  chmod 755 $TMPDIR/a.out
  stdout_actual=$(./run-on-linux.sh $TMPDIR/a.out)
  actual="$?"
  
  if [ "$actual" != "$expected" ]; then
    printf "\033[31m[FAIL]\033[m %s => %s expected, but got %s\n" "$input" "$expected" "$actual"
    exit 1
  fi

  if [ "$stdout_expected" != "" ]; then
    if [ "$stdout_expected" != "$stdout_actual" ]; then
      printf "\033[31m[FAIL]\033[m %s => %s expected, but got %s\n" "$input" "$stdout_expected" "$stdout_actual"
      exit 1
    fi
    printf "\033[32m[PASS]\033[m %s => \033[32mstdout:\033[m %s\n" "$input" "$stdout_actual"
  fi

  printf "\033[32m[PASS]\033[m %s => %s\n" "$input" "$actual"
  rm -rf $TMPDIR
}

check() {
  check_inner "$@" &
}
check 8 "main() { return 8; }"
check 27 "main() { return 27; }"
check 3 "main() { return 1+2; }"
check 2 "main() { return 5-3; }"
check 6 "main() { return 3+5-2; }"
check 2 "main() { return 5 - 3; }"
check 7 "main() { return 3*4-5; }"
check 9 "main() { return 5*6-3*7; }"
check 50 "main() { return 9 *8  - 7*  6 + 5*  4*1; }"
check 105 "main() { return 5*(6-3)*7; }"
check 2 "main() { return (6+8)/7; }"
check 9 "main() { return (9+3)/(10-8)+3; }"
check 3 "main() { return +3; }"
check 4 "main() { return -3+7; }"
check 12 "main() { return -3*-4; }"
check 1 "main() { return 4+-3; }"
check 1 "main() { return 3<=4; }"
check 1 "main() { return 4<=4; }"
check 0 "main() { return 4<4; }"
check 0 "main() { return 4>5; }"
check 1 "main() { return 3+4>5; }"
check 0 "main() { return 3+4==5; }"
check 1 "main() { return 3+4!=5*6; }"
check 0 "main() { return 0<=1>2; }"
check 1 "main() { return 0<=(1>2); }"
check 1 "main() { return 1<2==4>1; }"
check 4 "main() { 3; return 4; }"
check 8 "main() { 42-5; return 4+4; }"
check 8 "main() { 45<3; 25>=4; return 4+4; }"
check 7 "main() { a = 7; return a; }"
check 7 "main() { a = 3; b = 4; return a + b; }"
check 7 "main() { c = 28; b = 4; return c / b; }"
check 7 "main() { a = 3; b = a + 1; return a + b; }"
check 7 "main() { a = b = 7; return a; }"
check 7 "main() { a = b = 7; return b; }"
check 7 "main() { hoge = foo = 7; return hoge; }"
check 14 "main() { hoge = foo = 7; return foo + hoge; }"
check 7 "main() { return 7; return 3; }"
check 7 "main() { a = 3; b = 4; return a + b; b = 7; return a + b; }"
check 7 "main() { if(1)a=7;else a=10; return a; }"
check 10 "main() { if(0)a=7;else a=10; return a; }"
check 7 "main() { a=10;if(1)a=7;return a; }"
check 10 "main() { a=10;if(0)a=7;return a; }"
check 16 "main() { a=2;while(a<10)a=a*a;return a; }"
check 26 "main() { for(a=1;a<10;a=a+1)a=a*a;return a; }"
check 3 "main() { a = 3; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c; }"
check 12 "main() { a = 0; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c; }"
check 3 "main() { a = 0; b = 0; c = 3; if (a) if (b) { c = 2; } else { c = 7; } return c; }"
check 7 "main() { a = 0; b = 0; c = 3; if (a) {if (b) { c = 2; }} else { c = 7; } return c; }"
check 3 "main() { return __builtin_three(); }"
check 6 "main() { return __builtin_three()+__builtin_three(); }"
check 4 "main() { return __builtin_three()+1; }"
check 4 "main() { return 1+__builtin_three(); }"
check 1 "main() { __builtin_putchar(65); return 1; }" "A"
check 6 "one() { return 1; } two() { return one() + 1; } main() { return one() + two() + __builtin_three(); }"
check 1 "five() { return 5; } eleven() { return five() * 2 + 1; } main() { __builtin_putchar(__builtin_three() * eleven()); return 1; }" "!"
check 3 "three(x) { return 3; } main() { return three(6); }"
check 6 "id(x) { return x; } main() { return id(6); }"
check 6 "add(x, y, z) { return x + y + z; } main() { return add(1, 2, __builtin_three()); }"
check 0 "if_non0(n) { if (n) { __builtin_putchar(n + 48); } else { __builtin_putchar(32); } return 0; } print(n) { for(h=0;n>=100;h=h+1){n=n-100;} if_non0(h); for(t=0;n>=10;t=t+1){n=n-10;} if_non0(t); __builtin_putchar(n + 48); return 0; } printcomma(n) { print(n); __builtin_putchar(44); return n; } main() { a = 2; b = 1; while(a<127) { c = printcomma(a) + b; a = b; b = c; } return 0; }" "  2,  1,  3,  4,  7, 11, 18, 29, 47, 76,123,"
check 0 "print(n) { for (t = 0; n >= 10; t = t + 1) { n = n - 10; } if (t) { __builtin_putchar(t + 48); } else { __builtin_putchar(32); } __builtin_putchar(n + 48); return 0; } printcomma(n) { print(n); __builtin_putchar(44); return n; } main() { a = 0; b = 1; while (a < 100) { c = printcomma(a) + b; a = b; b = c; } return 0; }" " 0, 1, 1, 2, 3, 5, 8,13,21,34,55,89,"

check 55 "fib(n) { if(n == 0){ return 0; } else if(n == 1) { return 1; } return fib(n-1) + fib(n-2); } main() { return fib(10); }"
check 0 "if_non0(n) { if (n) { __builtin_putchar(n + 48); } else { __builtin_putchar(32); } return 0; } print(n) { for(h=0;n>=100;h=h+1){n=n-100;} if_non0(h); for(t=0;n>=10;t=t+1){n=n-10;} if_non0(t); __builtin_putchar(n + 48); return 0; } printcomma(n) { print(n); __builtin_putchar(44); return n; } fib(n) { if(n == 0){ return 0; } else if(n == 1) { return 1; } return fib(n-1) + fib(n-2); } main() { for(n=0;n<17;n=n+1) {printcomma(fib(n));} return 0; }" "  0,  1,  1,  2,  3,  5,  8, 13, 21, 34, 55, 89,144,233,377,610,987,"
# `%` ??? `?` ??? `++` ??? `-=` ???????????????????????????`255` ???????????????
# check 0 "if_non0(n) { __builtin_putchar(n ? n + 48 : 32); return 0; } print(n) { if_non0(n / 100); if_non0((n / 10) % 10); __builtin_putchar((n % 10) + 48); return 0; } main() { a = 0; b = 1; for(;a<255;) { print(a); __builtin_putchar(44); c = a+b; a = b; b = c; } return 0; }" "  0,  1,  1,  2,  3,  5,  8, 13, 21, 34, 55, 89,144,233,"

for job in `jobs -p`
do
    wait $job
done

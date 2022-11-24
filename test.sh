#!/bin/bash

check() {
  expected="$1"
  input="$2"

  cargo run -- "$input"
  chmod 755 ./a.out
  ./run-on-linux.sh ./a.out
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo -e "\033[32m[PASS]\033[m $input => $actual"
  else
    echo -e "\033[31m[FAIL]\033[m $input => $expected expected, but got $actual"
    exit 1
  fi
}
check 8 "return 8;"
check 27 "return 27;"
check 3 "return 1+2;"
check 2 "return 5-3;"
check 6 "return 3+5-2;"
check 2 "return 5 - 3;"
check 7 "return 3*4-5;"
check 9 "return 5*6-3*7;"
check 50 "return 9 *8  - 7*  6 + 5*  4*1;"
check 105 "return 5*(6-3)*7;"
check 2 "return (6+8)/7;"
check 9 "return (9+3)/(10-8)+3;"
check 3 "return +3;"
check 4 "return -3+7;"
check 12 "return -3*-4;"
check 1 "return 4+-3;"
check 1 "return 3<=4;"
check 1 "return 4<=4;"
check 0 "return 4<4;"
check 0 "return 4>5;"
check 1 "return 3+4>5;"
check 0 "return 3+4==5;"
check 1 "return 3+4!=5*6;"
check 0 "return 0<=1>2;"
check 1 "return 0<=(1>2);"
check 1 "return 1<2==4>1;"
check 4 "3; return 4;"
check 8 "42-5; return 4+4;"
check 8 "45<3; 25>=4; return 4+4;"
check 7 "a = 7; return a;"
check 7 "a = 3; b = 4; return a + b;"
check 7 "c = 28; b = 4; return c / b;"
check 7 "a = 3; b = a + 1; return a + b;"
check 7 "a = b = 7; return a;"
check 7 "a = b = 7; return b;"
check 7 "hoge = foo = 7; return hoge;"
check 14 "hoge = foo = 7; return foo + hoge;"
rm a.out

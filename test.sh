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
check 8 8
check 27 27
check 3 "1+2"
check 2 "5-3"
check 6 "3+5-2"
check 2 "5 - 3"
check 7 "3*4-5"
check 9 "5*6-3*7"
check 50 "9 *8  - 7*  6 + 5*  4*1"
check 105 "5*(6-3)*7"
check 2 "(6+8)/7"
check 9 "(9+3)/(10-8)+3"
check 3 "+3"
check 4 "-3+7"
check 12 "-3*-4"
check 1 "4+-3"
check 1 "3<=4"
check 1 "4<=4"
check 0 "4<4"
check 0 "4>5"
check 1 "3+4>5"
check 0 "3+4==5"
check 1 "3+4!=5*6"
check 0 "0<=1>2"
check 1 "0<=(1>2)"
check 1 "1<2==4>1"
check 4 "3; 4"
check 8 "42-5; 4+4"
check 8 "45<3; 25>=4; 4+4"
check 7 "a = 7; a"
check 7 "a = 3; b = 4; a + b"
check 7 "c = 28; b = 4; c / b"
check 7 "a = 3; b = a + 1; a + b"
rm a.out

#!/bin/bash

cd $(dirname $0)
cargo build

check_inner() {
  TMPDIR=$(mktemp -d --tmpdir=testwork)
  expected="$1"
  input="$2"

  (cd $TMPDIR && ../../target/debug/c_to_elf_compiler "$input")
  chmod 755 $TMPDIR/a.out
  ./run-on-linux.sh $TMPDIR/a.out
  actual="$?"
  rm -rf $TMPDIR

  if [ "$actual" = "$expected" ]; then
    printf "\033[32m[PASS]\033[m $input => $actual\n"
  else
    printf "\033[31m[FAIL]\033[m $input => $expected expected, but got $actual\n"
    exit 1
  fi
}

check() {
  check_inner "$1" "$2" &
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
check 7 "return 7; return 3;"
check 7 "a = 3; b = 4; return a + b; b = 7; return a + b;"
check 7 "if(1)a=7;else a=10; return a;"
check 10 "if(0)a=7;else a=10; return a;"
check 7 "a=10;if(1)a=7;return a;"
check 10 "a=10;if(0)a=7;return a;"
check 16 "a=2;while(a<10)a=a*a;return a;"
check 26 "for(a=1;a<10;a=a+1)a=a*a;return a;"
check 3 "a = 3; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c;"
check 12 "a = 0; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c;"
check 3 "a = 0; b = 0; c = 3; if (a) if (b) { c = 2; } else { c = 7; } return c;"
check 7 "a = 0; b = 0; c = 3; if (a) {if (b) { c = 2; }} else { c = 7; } return c;"
check 3 "return __builtin_three();"
check 6 "return __builtin_three()+__builtin_three();"
check 4 "return __builtin_three()+1;"
check 4 "return 1+__builtin_three();"
for job in `jobs -p`
do
echo $job
    wait $job
done

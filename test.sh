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
  rm -rf $TMPDIR

  if [ "$actual" != "$expected" ]; then
    printf "\033[31m[FAIL]\033[m $input => $expected expected, but got $actual\n"
    exit 1
  fi

  if [ "$stdout_expected" != "" ]; then
    if [ "$stdout_expected" != "$stdout_actual" ]; then
      printf "\033[31m[FAIL]\033[m $input => $stdout_expected expected, but got $stdout_actual\n"
      exit 1
    fi
    printf "\033[32m[PASS]\033[m $input => \033[32mstdout:\033[m $stdout_actual\n"
  fi
  

  printf "\033[32m[PASS]\033[m $input => $actual\n"
}

check() {
  check_inner "$@" &
}
check 8 "__throw 8;"
check 27 "__throw 27;"
check 3 "__throw 1+2;"
check 2 "__throw 5-3;"
check 6 "__throw 3+5-2;"
check 2 "__throw 5 - 3;"
check 7 "__throw 3*4-5;"
check 9 "__throw 5*6-3*7;"
check 50 "__throw 9 *8  - 7*  6 + 5*  4*1;"
check 105 "__throw 5*(6-3)*7;"
check 2 "__throw (6+8)/7;"
check 9 "__throw (9+3)/(10-8)+3;"
check 3 "__throw +3;"
check 4 "__throw -3+7;"
check 12 "__throw -3*-4;"
check 1 "__throw 4+-3;"
check 1 "__throw 3<=4;"
check 1 "__throw 4<=4;"
check 0 "__throw 4<4;"
check 0 "__throw 4>5;"
check 1 "__throw 3+4>5;"
check 0 "__throw 3+4==5;"
check 1 "__throw 3+4!=5*6;"
check 0 "__throw 0<=1>2;"
check 1 "__throw 0<=(1>2);"
check 1 "__throw 1<2==4>1;"
check 4 "3; __throw 4;"
check 8 "42-5; __throw 4+4;"
check 8 "45<3; 25>=4; __throw 4+4;"
check 7 "a = 7; __throw a;"
check 7 "a = 3; b = 4; __throw a + b;"
check 7 "c = 28; b = 4; __throw c / b;"
check 7 "a = 3; b = a + 1; __throw a + b;"
check 7 "a = b = 7; __throw a;"
check 7 "a = b = 7; __throw b;"
check 7 "hoge = foo = 7; __throw hoge;"
check 14 "hoge = foo = 7; __throw foo + hoge;"
check 7 "__throw 7; __throw 3;"
check 7 "a = 3; b = 4; __throw a + b; b = 7; __throw a + b;"
check 7 "if(1)a=7;else a=10; __throw a;"
check 10 "if(0)a=7;else a=10; __throw a;"
check 7 "a=10;if(1)a=7;__throw a;"
check 10 "a=10;if(0)a=7;__throw a;"
check 16 "a=2;while(a<10)a=a*a;__throw a;"
check 26 "for(a=1;a<10;a=a+1)a=a*a;__throw a;"
check 3 "a = 3; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } __throw b + c;"
check 12 "a = 0; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } __throw b + c;"
check 3 "a = 0; b = 0; c = 3; if (a) if (b) { c = 2; } else { c = 7; } __throw c;"
check 7 "a = 0; b = 0; c = 3; if (a) {if (b) { c = 2; }} else { c = 7; } __throw c;"
check 3 "__throw __builtin_three();"
check 6 "__throw __builtin_three()+__builtin_three();"
check 4 "__throw __builtin_three()+1;"
check 4 "__throw 1+__builtin_three();"
check 1 "__builtin_putchar(65); __throw 1;" "A"
for job in `jobs -p`
do
    wait $job
done

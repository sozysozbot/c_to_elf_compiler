#!/bin/bash

set -eu

cd $(dirname $0)
cargo build
jobs=()

check_inner() {
  TMPDIR=$(mktemp -d testwork/XXXXXX)
  expected="$1"
  input="$2"
  set +u
  stdout_expected="$3"
  set -u

  (cd $TMPDIR && ../../target/debug/c_to_elf_compiler <(echo "$input"))
  chmod 755 $TMPDIR/a.out
  set +e
  stdout_actual=$(./run-on-linux.sh $TMPDIR/a.out)
  actual="$?"
  set -e
  
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
  jobs_count=${#jobs[@]}
  if [ $jobs_count -gt 5 ]; then
    wait_jobs
  fi
  check_inner "$@" &
  jobs+=($!)
}

fail_count=0

wait_jobs() {
  for job in ${jobs[@]};
  do
    set +e
    wait $job
    if [ "$?" != "0" ]; then
      fail_count=$((fail_count + 1))
    fi
    set -e
  done
  jobs=()
}

check 8 "int main() { return 8; }"
check 27 "int main() { return 27; }"
check 3 "int main() { return 1+2; }"
check 2 "int main() { return 5-3; }"
check 6 "int main() { return 3+5-2; }"
check 2 "int main() { return 5 - 3; }"
check 7 "int main() { return 3*4-5; }"
check 9 "int main() { return 5*6-3*7; }"
check 50 "int main() { return 9 *8  - 7*  6 + 5*  4*1; }"
check 105 "int main() { return 5*(6-3)*7; }"
check 2 "int main() { return (6+8)/7; }"
check 9 "int main() { return (9+3)/(10-8)+3; }"
check 3 "int main() { return +3; }"
check 4 "int main() { return -3+7; }"
check 12 "int main() { return -3*-4; }"
check 1 "int main() { return 4+-3; }"
check 1 "int main() { return 3<=4; }"
check 1 "int main() { return 4<=4; }"
check 0 "int main() { return 4<4; }"
check 0 "int main() { return 4>5; }"
check 1 "int main() { return 3+4>5; }"
check 0 "int main() { return 3+4==5; }"
check 1 "int main() { return 3+4!=5*6; }"
check 0 "int main() { return 0<=1>2; }"
check 1 "int main() { return 0<=(1>2); }"
check 1 "int main() { return 1<2==4>1; }"
check 4 "int main() { 3; return 4; }"
check 8 "int main() { 42-5; return 4+4; }"
check 8 "int main() { 45<3; 25>=4; return 4+4; }"
check 7 "int main() { int a; a = 7; return a; }"
check 7 "int main() { int a; int b; a = 3; b = 4; return a + b; }"
check 7 "int main() { int c; int b; c = 28; b = 4; return c / b; }"
check 7 "int main() { int a; int b; a = 3; b = a + 1; return a + b; }"
check 7 "int main() { int a; int b; a = b = 7; return a; }"
check 7 "int main() { int a; int b; a = b = 7; return b; }"
check 7 "int main() { int hoge; int foo; hoge = foo = 7; return hoge; }"
check 14 "int main() { int hoge; int foo; hoge = foo = 7; return foo + hoge; }"
check 7 "int main() { return 7; return 3; }"
check 7 "int main() { int a; int b; a = 3; b = 4; return a + b; b = 7; return a + b; }"
check 7 "int main() { int a; if(1)a=7;else a=10; return a; }"
check 10 "int main() { int a; if(0)a=7;else a=10; return a; }"
check 7 "int main() { int a; a=10;if(1)a=7;return a; }"
check 10 "int main() { int a; a=10;if(0)a=7;return a; }"
check 16 "int main() { int a; a=2;while(a<10)a=a*a;return a; }"
check 26 "int main() { int a; for(a=1;a<10;a=a+1)a=a*a;return a; }"
check 3 "int main() { int a; int b; int c; a = 3; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c; }"
check 12 "int main() {int a; int b; int c; a = 0; if (a) { b = 1; c = 2; } else { b = 5; c = 7; } return b + c; }"
check 3 "int main() { int a; int b; int c; a = 0; b = 0; c = 3; if (a) if (b) { c = 2; } else { c = 7; } return c; }"
check 7 "int main() { int a; int b; int c; a = 0; b = 0; c = 3; if (a) {if (b) { c = 2; }} else { c = 7; } return c; }"
check 3 "int main() { return __builtin_three(); }"
check 6 "int main() { return __builtin_three()+__builtin_three(); }"
check 4 "int main() { return __builtin_three()+1; }"
check 4 "int main() { return 1+__builtin_three(); }"
check 1 "int main() { __builtin_putchar(65); return 1; }" "A"
check 6 "int one() { return 1; } int two() { return one() + 1; } int main() { return one() + two() + __builtin_three(); }"
check 1 "int five() { return 5; } int eleven() { return five() * 2 + 1; } int main() { __builtin_putchar(__builtin_three() * eleven()); return 1; }" "!"
check 3 "int three(int x) { return 3; } int main() { return three(6); }"
check 6 "int id(int x) { return x; } int main() { return id(6); }"
check 6 "int add(int x, int y, int z) { return x + y + z; } int main() { return add(1, 2, __builtin_three()); }"
check 0 "int if_non0(int n) { if (n) { __builtin_putchar(n + 48); } else { __builtin_putchar(32); } return 0; } int print(int n) { int h; int t; for(h=0;n>=100;h=h+1){n=n-100;} if_non0(h); for(t=0;n>=10;t=t+1){n=n-10;} if_non0(t); __builtin_putchar(n + 48); return 0; } int printcomma(int n) { print(n); __builtin_putchar(44); return n; } int main() { int a; int b; int c; a = 2; b = 1; while(a<127) { c = printcomma(a) + b; a = b; b = c; } return 0; }" "  2,  1,  3,  4,  7, 11, 18, 29, 47, 76,123,"
check 0 "int print(int n) { int t; for (t = 0; n >= 10; t = t + 1) { n = n - 10; } if (t) { __builtin_putchar(t + 48); } else { __builtin_putchar(32); } __builtin_putchar(n + 48); return 0; } int printcomma(int n) { print(n); __builtin_putchar(44); return n; } int main() { int a; int b; int c; a = 0; b = 1; while (a < 100) { c = printcomma(a) + b; a = b; b = c; } return 0; }" " 0, 1, 1, 2, 3, 5, 8,13,21,34,55,89,"

check 55 "int fib(int n) { if(n == 0){ return 0; } else if(n == 1) { return 1; } return fib(n-1) + fib(n-2); } int main() { return fib(10); }"
check 0 "int if_non0(int n) { if (n) { __builtin_putchar(n + 48); } else { __builtin_putchar(32); } return 0; } int print(int n) { int h; int t; for(h=0;n>=100;h=h+1){n=n-100;} if_non0(h); for(t=0;n>=10;t=t+1){n=n-10;} if_non0(t); __builtin_putchar(n + 48); return 0; } int printcomma(int n) { print(n); __builtin_putchar(44); return n; } int fib(int n) { if(n == 0){ return 0; } else if(n == 1) { return 1; } return fib(n-1) + fib(n-2); } int main() { int n; for(n=0;n<17;n=n+1) {printcomma(fib(n));} return 0; }" "  0,  1,  1,  2,  3,  5,  8, 13, 21, 34, 55, 89,144,233,377,610,987,"
# `%` と `?` と `++` と `-=` 演算子が未実装で、`255` も書けない
# check 0 "if_non0(n) { __builtin_putchar(n ? n + 48 : 32); return 0; } print(n) { if_non0(n / 100); if_non0((n / 10) % 10); __builtin_putchar((n % 10) + 48); return 0; } int main() { a = 0; b = 1; for(;a<255;) { print(a); __builtin_putchar(44); c = a+b; a = b; b = c; } return 0; }" "  0,  1,  1,  2,  3,  5,  8, 13, 21, 34, 55, 89,144,233,"

check 3 "int main() { int x; int *y; x = 3; y = &x; return *y; }"
check 3 "int main() { int x; int *y; y = &x; *y = 3; return x; }"
check 3 "int main() { int x; int *y; y = &x; x = 3; return *y; }"
check 3 "int main() { int x; *&x = 3; return x; }"

check 1 "int main() { int *p; p = __builtin_alloc4(1, 2, 4, 8); return *p; }"
check 7 "int main() { int *p; p = __builtin_alloc4(7, 2, 4, 8); return *p; }"
check 11 "int main() { int *p; p = __builtin_alloc4(11, 2, 4, 8); return *p; }"

check 1 "int main() { int *p; p = __builtin_alloc4(2, 1, 4, 8); return *(p+1); }"
check 7 "int main() { int *p; p = __builtin_alloc4(2, 7, 4, 8); return *(p+1); }"
check 11 "int main() { int *p; p = __builtin_alloc4(2, 11, 4, 8); return *(p+1); }"

check 1 "int main() { int *p; p = __builtin_alloc4(2, 4, 1, 8); return *(p+2); }"
check 7 "int main() { int *p; p = __builtin_alloc4(2, 4, 7, 8); return *(p+2); }"
check 11 "int main() { int *p; p = __builtin_alloc4(2, 4, 11, 8); return *(p+2); }"

check 1 "int main() { int *p; p = __builtin_alloc4(2, 4, 8, 1); return *(p+3); }"
check 7 "int main() { int *p; p = __builtin_alloc4(2, 4, 8, 7); return *(p+3); }"
check 11 "int main() { int *p; p = __builtin_alloc4(2, 4, 8, 11); return *(p+3); }"

check 15 "int main() { int *p; p = __builtin_alloc4(1, 2, 4, 8); return *p + *(p+1) + *(p+2) + *(p+3); }"

check 15 "int main() { int *p; p = __builtin_alloc4(3, 3, 3, 3); *(p+3) = 8; *(p+2) = 4; *(p+1) = 2; *p = 1; return *p + *(p+1) + *(p+2) + *(p+3); }"

check 4 "int main() { return sizeof(1); }"
check 8 "int main() { int x; return sizeof(&x); }"
check 4 "int main() { int x; return sizeof(sizeof(&x)); }"
check 4 "int main() { return sizeof 1; }"
check 4 "int main() { return sizeof(int); }"
check 8 "int main() { return sizeof(int*); }"
check 8 "int main() { return sizeof(int**); }"

check 20 "int main() { int arr[5]; return sizeof(arr); }"
check 40 "int main() { int arr[5][2]; return sizeof(arr); }"

check 3 "int main() { int a[2]; int *p; *a = 1; *(a + 1) = 2; p = a; return *p + *(p + 1); }"

check 2 "int main() { int a[20]; int *p; *(a + 9) = 2; p = a; return *(p + 9); }"

check 3 "int main() { int a[2]; int *p; a[0] = 1; a[1] = 2; p = a; return p[0] + a[1]; }"
check 2 "int main() { int a[20]; int *p; a[9] = 2; p = a; return p[9]; }"

check 20 "int arr[5]; int main() { return 20; }"
check 20 "int *arr[5][2]; int main() { return 20; }"
check 20 "int ***arr[3][5][2]; int main() { return 20; }"

check 20 "int arr[5]; int main() { return sizeof arr; }"
check 48 "int *arr[3][2]; int main() { return sizeof arr; }"
check 20 "int arr[7]; int main() { int arr[5]; return sizeof arr; }"

check 3 "int main() { char x[3]; int y; x[0] = -1; x[1] = 2; y = 4; return x[0] + y; }"
check 3 "int main() { char x; char y; x = 1; y = 2; x = x + y; return x; }"

# integral promotion
check 4 "int main() { char a; return sizeof((a+a)); }"
check 4 "int main() { char a; return sizeof((+a)); }"
check 4 "int main() { char a; return sizeof(a+a); }"

check 2 "int main() { return sizeof(\"a\"); }"

# single-line comment
check 8 "int main() { return 8; } // foo bar"
TESTCASE=$'int main() // foo\n{ // bar\n  return 8; // baz\n} // quux\n'
echo "$TESTCASE"
check 8 "$TESTCASE"

# multi-line comment
check 8 "int main() { return /*foo*/ 8; }"
check 8 "int main() { return /*/ */ 8; }"
TESTCASE2=$'int main() /* foo\n */ { // bar\n  return 8; // baz\n} // quux\n'
echo "$TESTCASE2"
check 8 "$TESTCASE2"

# increment and decrement operators
check 44 "int main() { int a; int b; a = 3; b = ++a; return b * 10 + a; }"
check 33 "int main() { int a; int b; a = 4; b = --a; return b * 10 + a; }"
check 34 "int main() { int a; int b; a = 3; b = a++; return b * 10 + a; }"
check 43 "int main() { int a; int b; a = 4; b = a--; return b * 10 + a; }"

# add-assign and sub-assign operators
check 7 "int main() { int a; a = 3; a += 4; return a; }"
check 3 "int main() { int a; a = 7; a -= 4; return a; }"
check 77 "int main() { int a; int b; a = 3; b = (a += 4); return b * 10 + a; }"
check 33 "int main() { int a; int b; a = 7; b = (a -= 4); return b * 10 + a; }"

# char literals
check 97 "int main() { char a; a = 'a'; return a; }"
check 65 "int main() { char a; a = 'A'; return a; }"
check 10 "int main() { char a; a = '\\n'; return a; }"
check 92 "int main() { char a; a = '\\\\'; return a; }"
check 39 "int main() { char a; a = '\\''; return a; }"

# logical not
check 1 "int main() { int a; a = 0; return !a; }"
check 0 "int main() { int a; a = 42; return !a; }"
check 0 "int main() { int *p; int a; p = &a; return !p; }"

# _Alignof
check 4 "int main() { return _Alignof(int); }"
check 8 "int main() { return _Alignof(int*); }"
check 8 "int main() { return _Alignof(int**); }"
check 1 "int main() { return _Alignof(char); }"

# struct definitions & sizeof & _Alignof
check 4 "struct S { int a; }; int main() { return sizeof(struct S); }"
check 8 "struct S { int a; int b; }; int main() { return sizeof(struct S); }"
check 12 "struct S { int a; int b; int c; }; int main() { return sizeof(struct S); }"
check 8 "struct S { int a; char b; }; int main() { return sizeof(struct S); }"
check 8 "struct S { char b; int a; }; int main() { return sizeof(struct S); }"
check 2 "struct S { char b; char a; }; int main() { return sizeof(struct S); }"
check 1 "struct S { char a; }; int main() { return sizeof(struct S); }"
check 16 "struct S { int *a; char b; }; int main() { return  sizeof(struct S); }"

check 4 "struct S { int a; }; int main() { return _Alignof(struct S); }"
check 4 "struct S { int a; int b; }; int main() { return _Alignof(struct S); }"
check 4 "struct S { int a; int b; int c; }; int main() { return _Alignof(struct S); }"
check 4 "struct S { int a; char b; }; int main() { return  _Alignof(struct S); }"
check 4 "struct S { char b; int a; }; int main() { return  _Alignof(struct S); }"
check 1 "struct S { char b; char a; }; int main() { return _Alignof(struct S); }"
check 1 "struct S { char a; }; int main() { return _Alignof(struct S); }"
check 8 "struct S { int *a; char b; }; int main() { return  _Alignof(struct S); }"

# arrow operator
check 42 "struct S { int a; int b; }; int main() { struct S s; (&s)->a = 42; return (&s)->a; }"
check 3 "struct S { int a; int b; }; int main() { struct S s; struct S *p; (&s)->a = 1; (&s)->b = 2; p = &s; return p->a + p->b; }"

# dot operator
check 42 "struct S { int a; int b; }; int main() { struct S s; s.a = 42; return s.a; }"
check 3 "struct S { int a; int b; }; int main() { struct S s; struct S *p; s.a = 1; s.b = 2; p = &s; return p->a + p->b; }"

check 42 "struct S { int a; int b; }; struct S2 { int a; int b; }; int main() { struct S s; s.a = 42; return s.a; }"


# struct inside struct
check 24 "struct T { int *p; char a; }; struct S { int a; struct T t; }; int main() { return sizeof(struct S); }"
check 27 "struct T { int *p; char a; }; struct S { int a; struct T t; }; int main() { struct S s; int k; k = 4; s.a = 3; s.t.a = 20; s.t.p = &k; return s.a + s.t.a + *s.t.p; }"

# write through a pointer across a function call
check 3 "int update(int *p) { *p = 3; return 1; }  int main() { int a; a = 42; update(&a); return a; }"


# logical and
check 1 "int main() { int a; int b; a = 1; b = 2; return a && b; }"
check 0 "int main() { int a; int b; a = 0; b = 2; return a && b; }"
check 0 "int main() { int a; int b; a = 1; b = 0; return a && b; }"
check 0 "int main() { int a; int b; a = 0; b = 0; return a && b; }"

# short-circuit evaluation of logical and
check 3 "int update(int *p) { *p = 3; return 0; }  int main() { int a; int b; a = 42; b = 1 && update(&a); return b * 10 + a; }"
check 13 "int update(int *p) { *p = 3; return 1; }  int main() { int a; int b; a = 42; b = 1 && update(&a); return b * 10 + a; }"
check 42 "int update(int *p) { *p = 3; return 0; }  int main() { int a; int b; a = 42; b = 0 && update(&a); return b * 10 + a; }"
check 42 "int update(int *p) { *p = 3; return 1; }  int main() { int a; int b; a = 42; b = 0 && update(&a); return b * 10 + a; }"



# logical or
check 1 "int main() { int a; int b; a = 1; b = 2; return a || b; }"
check 1 "int main() { int a; int b; a = 1; b = 0; return a || b; }"
check 1 "int main() { int a; int b; a = 0; b = 1; return a || b; }"
check 0 "int main() { int a; int b; a = 0; b = 0; return a || b; }"

# short-circuit evaluation of logical or
check 52 "int update(int *p) { *p = 3; return 0; }  int main() { int a; int b; a = 42; b = 1 || update(&a); return b * 10 + a; }"
check 52 "int update(int *p) { *p = 3; return 1; }  int main() { int a; int b; a = 42; b = 1 || update(&a); return b * 10 + a; }"
check 3 "int update(int *p) { *p = 3; return 0; }  int main() { int a; int b; a = 42; b = 0 || update(&a); return b * 10 + a; }"
check 13 "int update(int *p) { *p = 3; return 1; }  int main() { int a; int b; a = 42; b = 0 || update(&a); return b * 10 + a; }"

# variable declaration that are not necessarily at the start of a function
check 3 "int main() { int a; a = 7; int b; b = 4; return a - b; }"
check 1 "int main() { int a; a = 1; int b; b = 2; return a || b; }"

# variable declaration with initialization
check 3 "int main() { int a = 7; int b = 4; return a - b; }"
check 1 "int main() { int a = 1; int b = 2; return a || b; }"

# void type
check 3 "void f(int *p) { *p = 3; return; } int main() { int a = 30; f(&a); return a; }"
check 3 "void f(int *p) { *p = 3; } int main() { int a = 30; f(&a); return a; }"

wait_jobs
if [ $fail_count -gt 0 ]; then
  echo "$fail_count tests failed"
  exit 1
fi

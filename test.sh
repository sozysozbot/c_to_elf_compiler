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

check 3 "int three(void) { return 3; } int main() { return three(); }"

# assigning an int to a char through a pointer
# 7*7*7 = 343 ≡ 87 = 29 x 3 (mod 256)
check 29 "int main() { int a; a = 7; a = a * a; a = a * 7; char b; b = a; return b / 3; }"
check 7 "int main() { int a = 7; char c; char *p = &c; *p = a; return *p; }"
check 87 "int main() { int a = 7; a = a * 49; char c; char *p = &c; *p = a; return *p; }"
check 29 "int main() { int a = 7; a = a * 49; char c; char *p = &c; *p = a; return *p / 3; }"

# void* automatically works
check 29 "int main() { int a; a = 7; a = a * a; a = a * 7; int *p; p = &a; char *r; r = p; return *r / 3; }"
check 29 "int main() { int a; a = 7; a = a * a; a = a * 7; int *p; p = &a; void *q; q = p; char *r; r = q; return *r / 3; }"

# null pointer
check 1 "int main() { int *p; p = 0; if (p) { return 0; } else { return 1; } }"
# clearing the lower 32 bits does not make it a null pointer
check 1 "int main() { int a; int *p; p = &a; int **pp; pp = &p; void *q; q = pp; int *r; r = q; *r = 0; if (p) { return 1; } else { return 0; } }"

check 42 "int addition(int x, int y) { return x + y; } int main() { return 42; }"
check 42 "int id(int x) { return x; } int main() { return 42; }"

# scope chain
check 3 "int main() { int a; a = 3; { int a; a = 4; } return a; }"
check 3 "int main() { int a; a = 3; { int a; a = 4; } { int a; a = 5; } return a; }"
check 4 "int main() { int a = 3; int b; { int a = 4; b = a; } return b; }"
check 43 "int main() { int a = 3; int b; { int a = 4; b = a; } return b * 10 + a; }"

# for loop with an initialization statement
check 45 "int main() { int b; for (int a = 0; a < 10; a++) { b = b + a; } return b; }"

# parameter with type char
check 3 "int foo(char a) { return a; } int main() { return foo(3); }"
check 29 "int foo(char a) { return a; } int main() { return foo(7*7*7) / 3; }"


# tests taken from hsjoihs-c-compiler


run_test() {
  id="$1"
  content="$2"
  expected="$3"
  check "$expected" "$content" 
}

run_test 371 'int main() { int a; if (1) { a = 3; } else { a = 7; } return a; }' 3
run_test 372 'void foo(int*p) {*p=3;} int main() { int a; if (0) { foo(&a); } else { a = 7; } return a; }' 7
run_test 373 'void foo(int*p) {*p=7;} int main() { int a; if (0) { a = 3; } else { foo(&a); } return a; }' 7
run_test 376 'int main() { int a; return sizeof a; }' 4
run_test 377 "int main() { return sizeof 'C'; }" 4
run_test 378 "int main() { char a; return sizeof a; }" 1
run_test 379 "int main() { char a; return sizeof +a; }" 4
run_test 380 'int main() { int *a; return sizeof a; }' 8
run_test 381 'int main() { int *a; return sizeof (a+0); }' 8
run_test 382 'int main() { int a[2][3]; return sizeof a; }' 24
run_test 383 'int main() { int a[2][3]; return sizeof (a+0); }' 8
run_test 357 'int main() {return 0;} //nfsjdgkssfdvc' 0

run_test 3440 'int main(){char a[45]; return a + 3 - a; }' 3

run_test 352 'struct A {int a; int b; int c;}; int main(){struct A a[5]; return a + 3 - a;}' 3


run_test 204 'int main(){int *p; p = 0; if(p) {return 4; } return 174;}' 174
run_test 205 'int main(){int *p; int a; p = &a; if(p) {return 4; } return 174;}' 4
run_test 206 'int main(){int *p; int a; p = &a; return p && &p;}' 1
run_test 207 'int main(){int *p; int a; p = &a; return p || &p;}' 1
run_test 210 'int main(void){return 174;}' 174
run_test 211 'int main(void){void *p; p = 0; p = p; return 174;}' 174

run_test 291 'int *foo(){return 0;} int main(){int *p = foo(); if (p == 0) {return 174;} else {return 1;} }' 174

run_test 287 'int main(){int a[5];int *p = a;if (p == 0) {return 174;} else {return 1;}}' 1
run_test 288 'int main(){int a[5];int *p = 0;if (p == 0) {return 174;} else {return 1;}}' 174
run_test 289 'int main(){int a[5];int *p = 0;if (p != 0) {return 174;} else {return 1;}}' 1
run_test 290 'int main(){int a[5];int *p = a;if (p != 0) {return 174;} else {return 1;}}' 174


run_test 274 'int main(){int a[5];int *p = a;int *q = a+3;if (p < q) {return 174;} else {return 1;}}' 174
run_test 275 'int main(){int a[5];int *p = a;int *q = a+3;if (p > q) {return 174;} else {return 1;}}' 1
run_test 276 'int main(){int a[5];int *p = a;int *q = a+3;if (p <= q) {return 174;} else {return 1;}}' 174
run_test 277 'int main(){int a[5];int *p = a;int *q = a+3;if (p >= q) {return 174;} else {return 1;}}' 1
run_test 278 'int main(){int a[5];int *p = a;int *q = a;if (p < q) {return 174;} else {return 1;}}' 1
run_test 279 'int main(){int a[5];int *p = a;int *q = a;if (p > q) {return 174;} else {return 1;}}' 1
run_test 280 'int main(){int a[5];int *p = a;int *q = a;if (p <= q) {return 174;} else {return 1;}}' 174
run_test 281 'int main(){int a[5];int *p = a;int *q = a;if (p >= q) {return 174;} else {return 1;}}' 174
run_test 282 'int main(){int a[5];int *p = a;int *q = a;if (p == q) {return 174;} else {return 1;}}' 174
run_test 283 'int main(){int a[5];int *p = a;int *q = a;if (p != q) {return 174;} else {return 1;}}' 1
run_test 284 'int main(){int a[5];int *p = a;int *q = a+3;if (p == q) {return 174;} else {return 1;}}' 1
run_test 285 'int main(){int a[5];int *p = a;int *q = a+3;if (p != q) {return 174;} else {return 1;}}' 174

run_test 261 'int main(void){int a = 5; return 174;}' 174
run_test 262 'int main(void){int u = 0; for(int a = 0; a < 10; a++){ u += a; } return 174+u-45;}' 174

run_test 252 'int main(void){int a = 5; int *p = &a; return 174;}' 174
run_test 253 'int main(void){int a = 4; int *p = &a; *p += 170; return a;}' 174
run_test 254 'int main(){int a; int *p = &a; *p = 2; int *q = &*p; *q = 174; return a;}' 174
run_test 255 'int main(){int a; int *p = &a; *p = 2; int *q = &(*p); *q = 174; return a;}' 174
run_test 256 'int main(){int x = 86;int *y = &x; return (*y) + x + 2;}' 174
run_test 257 'int main(){int x = 86;int *y = &x; return (*y) + (*y) + 2;}' 174
run_test 258 'int main(){int x = 86;int *y = &x;int **z = &y;return (*y) + (**z) + 2;}' 174
run_test 259 'int main(){int x = 86;int *y = &x;int **z = &y;return*y+**z+2;}' 174

run_test 133 'char foo(){char a; return a;} int main(){foo(); return 174;}' 174
run_test 134 'char foo(char *p){char a; return a;} int main(){char q; foo(&q); return 174;}' 174
run_test 135 'char foo(char *p){char a; a = 5; return a;} int main(){char q; foo(&q); return 174;}' 174
run_test 136 'int main(){char x[3]; x[0] = -1; x[1] = 2; int y; y = 4; return x[0] + y + 171;}' 174
run_test 137 'char foo(char *p){*p = 5; char a;a = 3; return a;} int main(){char q; char r; r = foo(&q); return 172-r+q;}' 174
run_test 139 'int foo(char a){int d;d = 3;char c;c = a+d;return c;} int main(){char f;f=3;return foo(f)*4+150;}' 174
run_test 140 'int foo(char a){int d;d = 3;char c;c = a+d;return c*4;} int main(){char f;f=3;return foo(f)+150;}' 174
run_test 143 'int foo(char a, char b){return 23;} int main(){char f;f=3;return foo(f,4)+151;}' 174
run_test 144 'int foo(char a, char b){return a*4+11;} int main(){char f;f=3;return foo(f,4)+151;}' 174
run_test 145 'int foo(char a, char b){return a*4+12;} int main(){char f;f=3;return foo(f,4)+150;}' 174
run_test 146 'int foo(char a, char b){return (a+3)*4;} int main(){char f;f=3;return foo(f,4)+150;}' 174
run_test 147 'int foo(char a, char b){char c;c = a+3;return c*4;} int main(){char f;f=3;return foo(f,4)+150;}' 174
run_test 148 'int foo(char a, char b){int d;d = 3;char c;c = a+d;return c*4;} int main(){char f;f=3;return foo(f,4)+150;}' 174
run_test 149 'int foo(char a, char b){int d;d = 3;char c;c = a+d;return c*b;} int main(){char f;f=3;return foo(f,4)+150;}' 174
run_test 156 'int main(){/**/return 123;}' 123
run_test 157 'int main(){/*u89g3wihu-@w3erolk*/ return (123);}' 123
run_test 161 'int main(){int a[10]; a[5] = 173; int b; b = a[5]++; return a[5]*!(a[5]-b-1);}' 174

run_test 001 'int main(){return 123;}' 123
run_test 002 'int main(){return (123);}' 123
run_test 003 'int main(){return ((((123))));}' 123
run_test 004 'int main(){return 123+51;}' 174
run_test 005 'int main(){return 123+56-5;}' 174
run_test 006 'int main(){return 175-(4-3);}' 174
run_test 007 'int main(){return 181-4-3;}' 174
run_test 009 'int main(){return 6*(3+7)-5*1;}' 55
run_test 018 'int main(){return +174;}' 174
run_test 019 'int main(){return -(1-175);}' 174
run_test 020 'int main(){23; 45+37; ((12-1)*75); return -(1-175);}' 174
run_test 021 'int main(){23; 45+37; return -(1-175); ((12-1)*75);}' 174


run_test 034 'int add_(int x, int y){4; return x+y;} int main(){3; return add_(87,87);}' 174
run_test 037 'int main() { return (3 && 2 && 5) + 173; }' 174
run_test 038 'int main() { return (3 && 2) + !(3 && 0) + !(0 && 3)+ !(0 && 0) + 170; }' 174
run_test 039 'int main() { return (3 || 2 || 5) + 173; }' 174
run_test 040 'int main() { return (3 || 2) + (3 || 0) + (0 || 3)+ !(0 || 0) + 170; }' 174
run_test 041 'int main() {int a; a = 3; a += 5;  return a + 166; }' 174
run_test 042 'int main() {int a; int b; a = 3; b = (a += 5);  return a + b + 158; }' 174
run_test 046 'int foo(){ return 2;} int main() {int a; int b; int c; a = 3;b = 5;c = 2;if(a) {b = foo();} else { }    return 172+b;}' 174
run_test 047 'int foo(){ return 2;} int main() {int a; int b; int c; a = 3;b = 5;c = 2;if(a) {b = foo();}   return 172+b;}' 174
run_test 048 'int foo(){ return 2;} int bar(){ return 7;} int main() {int a; int b; int c; a = 3;b = 5;c = 2;if(a) {b = foo();} else { c = bar();}    return 172+b;}' 174
run_test 049 'int foo(){ return 2;} int bar(){ return 7;} int main() {int a; int b; int c; a = 0;b = 5;c = 2;if(a) {b = foo();} else { c = bar();}    return 162+b+c;}' 174
run_test 050 'int foo(){ return 2;} int bar(){ return 7;} int main() {int a; int b; int c; a = 3;b = 5;c = 2;if(a) if(0) { b = foo(); } else {  c = bar(); }    return 162+b+c;}' 174
run_test 051 'int foo(){ return 2;} int bar(){ return 7;} int main() {int a; int b; int c; a = 3;b = 5;c = 2;if(a) if(0)b=foo();else c = bar();return 162+b+c;}' 174
run_test 052 'int main() {int a; a = 4; if(1){return 170+a; a = 7; }else{return 170-a; a = 9;} a = 5; return a;}' 174
run_test 056 'int foo(){return 3;} int main() {int a; a = 0;while(a == foo()) {a = 3;}return 174;}' 174
run_test 057 'int main(){int a; int b; a = 0; b = 0; while(a <= 10) {b += a; a += 1;}return b;}' 55
run_test 064 'int main(){int a; int b; a =-3; b=-6; return a*b*10+a+b+3;}' 174
run_test 074 'int main(){int a; int b; a=3; b=0; b+= ++a; return a*b*11-2;}' 174
run_test 075 'int main(){int a; int b; a=3; b=0; b+= a++; return !(b-3)+!(a-4)+172;}' 174
run_test 085 'int main(){int a; a = 174; {int a; a = 3;} return a;}' 174
run_test 086 'int main(){int a; a = 3; { a = 174;} return a;}' 174
run_test 087 'int main() {int *b; int a; a = 3; a += 5;  return a + 166; }' 174
run_test 088 'int main() {int *******b; int a; a = 3; a += 5;  return a + 166; }' 174
run_test 089 'int main() {int a; a = 174; int *b; b = &a; return a;}' 174
run_test 090 'int main(){int x;x = 86;int *y;y = &x; return (*y) + x + 2;}' 174
run_test 091 'int main(){int x;x = 86;int *y;y = &x; return (*y) + (*y) + 2;}' 174
run_test 092 'int main(){int x;x = 86;int *y;y = &x;int **z;z = &y;return (*y) + (**z) + 2;}' 174
run_test 093 'int main(){int x;x = 86;int *y;y = &x;int **z;z = &y;return*y+**z+2;}' 174
run_test 094 'int main() {int x;int *y;x = 3;y = &x;*y = 174;return x;}' 174
run_test 095 'int main() {int x;int *y;x = 3;y = &x;*y = 171;*y += 3;return x;}' 174
run_test 096 'int main(){int x; int y; int *z; int*a; z=&x; a=&y; *z=*a=87; return(x+y);}' 174
run_test 097 'int main(){int x; int *y; int **z; z = &y; *z = &x; *y = 174; return x;}' 174
run_test 098 'int foo(int* p){return 3;} int main(){int x; return 174;}' 174
run_test 099 'int foo(int* p){return *p;} int main(){int x; x = 174; return foo(&x);}' 174
run_test 100 'int foo(int* p){*p = 172; return *p+2;} int main(){int x; return foo(&x);}' 174
run_test 114 'int main(){int a[2][3]; return 174;}' 174
run_test 184 'struct A{int a; int b;}; int main(){ struct A a; return 174;}' 174
run_test 185 'struct A{int a; int b;}; int main(){ struct A a[10]; return 174;}' 174
run_test 186 'struct A{int a; int b;};  struct A a[10]; int main(){return 174;}' 174
run_test 192 'struct A{int a; int b;}; int main(){ return sizeof(struct A);}' 8
run_test 193 'struct A{int a; char c; char d; int b;}; int main(){ return sizeof(struct A);}' 12

run_test 190 'int main(){return sizeof(int);}' 4
run_test 191 'int main(){return sizeof(int*);}' 8
run_test 273 'int main(void){char a[5]; a[1] = 74; char *p = a + 2; return *--p;}' 74
run_test 267 'int main(void){char a[5]; a[1] = 74; char *p = a + 3; p -= 2; return *p;}' 74

check 174 'int main(void){int a[5]; a[3] = 174; int *p = a + 2; p = p + 1; return *p;}'
check 174 'int main(void){int a[5]; *(a + 3) = 174; int *p = a + 2; p = p + 1; return *p;}'

run_test 269 'int main(void){int a[5]; a[3] = 174; int *p = a + 2; p++; return *p;}' 174
run_test 270 'int main(void){int a[5]; a[3] = 174; int *p = a + 2; ++p; return *p;}' 174
run_test 271 'int main(void){int a[5]; a[3] = 174; int *p = a + 2; return *++p;}' 174
run_test 272 'int main(void){int a[5]; a[3] = 174; int *p = a + 3; return *p++;}' 174
run_test 264 'int main(void){int a[5]; a[3] = 174; int *p = a; p += 3; return *p;}' 174
run_test 266 'int main(void){int a[5]; a[1] = 174; int *p = a + 3; p -= 2; return *p;}' 174

run_test 176 'int main(){int a; int *p; p = &a; *p = 2; int *q; q = &*p; *q = 174; return a;}' 174
run_test 177 'int main(){int a; int *p; p = &a; *p = 2; int *q; q = &(*p); *q = 174; return a;}' 174
run_test 178 'char foo(char *p){char a; return a;} int main(){char q; foo(&(q)); return 174;}' 174

run_test 118 'int main(){int a[1]; int *p; p = a; *p=2; return 174;}' 174
run_test 119 'int main(){int a[1]; *(a+0)=2;return 174;}' 174

run_test 123 'int main(){int a[1][2];int *q;q = *a;return 174;}' 174
run_test 124 'int main(){int a[1][2];int *q;q = *a; *q=174; return **a;}' 174
run_test 125 'int main(){int a[86][2];int *q;q = *(a+1); *q=174; return **(a+1);}' 174
run_test 126 'int main(){int a[5][6];int *q;q = *(a+1); *(2+q)=174; return *(*(1+a)+2);}' 174

# run_test 101 'int *foo(int *p){*p = 4;return p;} int main(){int x;int *y;y = foo(&x); *y+= 170;return x;}' 174
# run_test 102 'int *foo(int *p){*p = 4;return p;} int main(){int x;int y;*foo(&x) += 170;return x;}' 174
# run_test 113 'int *foo(int *p){*p = 4;return p;} int main(){int x;int y; int **z; *foo(&x) += 170;return x;}' 174


wait_jobs
if [ $fail_count -gt 0 ]; then
  echo "$fail_count tests failed"
  exit 1
fi

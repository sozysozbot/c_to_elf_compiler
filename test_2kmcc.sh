cargo build
printf "\033[94m2kmcc ステップ1：整数1個をコンパイルする言語の作成\033[0m\n"
printf "\033[94m2kmcc Step 1: making a language that can compile a single integer\033[0m\n"
printf "===========テスト対象のコード===========\n"
cat 2kmcc_steps/step1.c
printf "\n===========Input===========\n"
printf "42"
printf "\n======Expected Output=====\n"
cat 2kmcc_steps/step1_expected_output.txt
printf "=========実行結果=========\n"
cd 2kmcc_steps/tmp
../../target/debug/c_to_elf_compiler ../step1.c
chmod 755 ./a.out
./a.out "42" > step1_actual_output.txt
diff -u ../step1_expected_output.txt step1_actual_output.txt || exit 1
printf "\033[92m2kmcc ステップ1 成功\033[0m\n"

cd ../../

printf "\033[94m-------------------------------------------------\033[0m\n"
printf "\033[94mステップ2：加減算のできるコンパイラの作成\033[0m\n"
printf "\033[94mStep 2: making a compiler that can add and subtract\033[0m\n"
printf "===========テスト対象のコード===========\n"
cat 2kmcc_steps/step2.c
printf "\n===========Input===========\n"
printf "111+10-42"
printf "\n======Expected Output=====\n"
cat 2kmcc_steps/step2_expected_output.txt
printf "=========実行結果=========\n"
cd 2kmcc_steps/tmp
../../target/debug/c_to_elf_compiler ../step2.c
chmod 755 ./a.out
./a.out "111+10-42" > step2_actual_output.txt
diff -u ../step2_expected_output.txt step2_actual_output.txt || exit 1
printf "\033[92m2kmcc ステップ2 成功\033[0m\n"
#!/bin/zsh

desc='请输入要运行的命令:
      md.sh build     ---> 构建pb文件
      md.sh clean     ---> 清理target文件
      md.sh run       ---> 运行服务'

pb_path='./src/proto'

if [ $# -lt 1 ]; then
    echo "$desc"
    exit 1
fi

case $1 in
"build")
#  echo "rm -rf $pb_path/*.rs"
#  rm -rf $pb_path/*.rs

  echo "cargo run --bin pb-build"
  sudo cargo run --bin pb-build || exit 2
#
#  echo 'ls $pb_path/*.rs | sed s/$*.rs//g | cut -d"/" -f3-4 | sed "s/pb\//mod /g"'
#  ls $pb_path/*.rs | sed s/$*.rs//g | cut -d"/" -f3-4 | sed "s/pb\//mod /g" > "$pb_path/mod.rs"

  ;;
"run")
  echo "cargo run --bin server -- run"
  sudo cargo run --bin server -- run
;;
"clean_task")
  echo "cargo run --bin server -- clean"
  sudo cargo run --bin server -- clean
;;
"clean")
  echo "cargo clean"
  sudo cargo clean
;;
esac
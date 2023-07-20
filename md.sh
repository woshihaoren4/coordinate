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
  echo "cargo run --bin coordinate -- run"
  sudo cargo run --bin coordinate -- run -c ./src/config/dev_config.toml
;;
"build_rel")
  echo "cargo build --release --bin=coordinate"
  sudo cargo build --release --bin=coordinate
;;
"clean_task")
  echo "cargo run --bin coordinate -- clean"
  sudo cargo run --bin coordinate -- clean
;;
"clean")
  echo "cargo clean"
  sudo cargo clean
;;
"build_docker")

if [ ! -e ".cargo/config.toml" ] ; then
  mkdir .cargo;touch .cargo/config.toml
fi

cat>".cargo/config.toml" <<EOF
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
EOF

sudo cargo build --release --bin=coordinate --target=x86_64-unknown-linux-musl
chmod +x target/x86_64-unknown-linux-musl/release/coordinate
tag="wdshihaoren/coordinate:v0.0.4"
docker build -f ./Dockerfile -t "$tag"  .
docker push "$tag"
rm -rf .cargo

;;
esac
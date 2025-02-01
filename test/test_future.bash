#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

res=$($com <<< 'a=(aaa bbb); bbb=eeee ; echo ${!a[1]}')
[ "$res" = "eeee" ] || err $LINENO

res=$($com <<< '[[ a =~ "." ]]')
[ $? -eq 1 ] || err $LINENO

echo $0 >> ./ok

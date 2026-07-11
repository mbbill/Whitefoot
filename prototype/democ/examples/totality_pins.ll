declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i64 @spin() nounwind memory(none) {
entry:
  %t1 = alloca i64
  %t2 = icmp eq i64 0, 0
  br i1 %t2, label %L4, label %L5
L4:
  br label %L6
L6:
  br label %L6
L7:
  br label %L3
L5:
  store i64 1, ptr %t1
  br label %L3
L3:
  %t8 = load i64, ptr %t1
  ret i64 %t8
}

define i64 @caller() nounwind memory(none) {
entry:
  %t1 = alloca i64
  %t2 = icmp eq i64 0, 0
  br i1 %t2, label %L4, label %L5
L4:
  %t6 = call i64 @spin()
  store i64 %t6, ptr %t1
  br label %L3
L5:
  store i64 2, ptr %t1
  br label %L3
L3:
  %t7 = load i64, ptr %t1
  ret i64 %t7
}

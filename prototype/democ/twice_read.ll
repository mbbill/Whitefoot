declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare void @llvm.trap()
define i32 @twice_read(ptr noalias readonly %a, ptr noalias %b) {
entry:
  %t1 = load i32, ptr %a
  %t2 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t1, i32 1)
  %t3 = extractvalue {i32, i1} %t2, 0
  %t4 = extractvalue {i32, i1} %t2, 1
  br i1 %t4, label %trap, label %t5
t5:
  store i32 %t3, ptr %b
  %t6 = load i32, ptr %a
  %t7 = load i32, ptr %a
  %t8 = add i32 %t6, %t7
  ret i32 %t8
trap:
  call void @llvm.trap()
  unreachable
}

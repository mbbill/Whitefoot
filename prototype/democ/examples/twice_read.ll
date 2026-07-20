declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i32 @twice_read(ptr %a, ptr %b) {
entry:
  %t9 = alloca i32
  %t1 = load i32, ptr %a
  %t2 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t1, i32 1)
  %t3 = extractvalue {i32, i1} %t2, 0
  %t4 = extractvalue {i32, i1} %t2, 1
  br i1 %t4, label %trap, label %L5
L5:
  store i32 %t3, ptr %b
  %t6 = load i32, ptr %a
  %t7 = load i32, ptr %a
  %t8 = add i32 %t6, %t7
  store i32 %t8, ptr %t9
  %t10 = load i32, ptr %t9
  ret i32 %t10
trap:
  call void @llvm.trap()
  unreachable
}

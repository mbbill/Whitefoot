declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()
declare i64 @llvm.fshl.i64(i64, i64, i64)

define void @accumulate(ptr noalias %acc, ptr noalias readonly %addend, i64 %n) {
entry:
  %t1 = alloca i64
  store i64 0, ptr %t1
  br label %L2
L2:
  %t4 = load i64, ptr %t1
  %t5 = icmp uge i64 %t4, %n
  br i1 %t5, label %L7, label %L8
L7:
  br label %L3
L8:
  br label %L6
L6:
  %t9 = load i64, ptr %acc
  %t10 = call i64 @llvm.fshl.i64(i64 %t9, i64 %t9, i64 1)
  %t11 = alloca i64
  store i64 %t10, ptr %t11
  %t12 = load i64, ptr %t11
  %t13 = load i64, ptr %addend
  %t14 = xor i64 %t12, %t13
  store i64 %t14, ptr %acc
  %t15 = load i64, ptr %t1
  %t16 = add i64 %t15, 1
  store i64 %t16, ptr %t1
  br label %L2
L3:
  ret void
}

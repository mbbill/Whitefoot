declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i32 @main() {
entry:
  %t1 = alloca i32
  store i32 0, ptr %t1
  %t2 = alloca i32
  store i32 0, ptr %t2
  br label %L3
L3:
  %t5 = load i32, ptr %t1
  %t6 = icmp sge i32 %t5, 5
  br i1 %t6, label %L8, label %L9
L8:
  br label %L4
L9:
  br label %L7
L7:
  %t10 = load i32, ptr %t2
  %t11 = load i32, ptr %t1
  %t12 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t10, i32 %t11)
  %t13 = extractvalue {i32, i1} %t12, 0
  %t14 = extractvalue {i32, i1} %t12, 1
  br i1 %t14, label %trap, label %L15
L15:
  store i32 %t13, ptr %t2
  %t16 = load i32, ptr %t1
  %t17 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t16, i32 1)
  %t18 = extractvalue {i32, i1} %t17, 0
  %t19 = extractvalue {i32, i1} %t17, 1
  br i1 %t19, label %trap, label %L20
L20:
  store i32 %t18, ptr %t1
  br label %L3
L4:
  %t21 = load i32, ptr %t2
  %t22 = icmp eq i32 %t21, 10
  br i1 %t22, label %L23, label %trap
L23:
  %t24 = alloca i32
  %t25 = load i32, ptr %t2
  %t26 = call {i32, i1} @llvm.smul.with.overflow.i32(i32 %t25, i32 2)
  %t27 = extractvalue {i32, i1} %t26, 0
  %t28 = extractvalue {i32, i1} %t26, 1
  br i1 %t28, label %L31, label %L30
L30:
  store i32 %t27, ptr %t24
  br label %L29
L31:
  ret i32 0
L29:
  %t32 = load i32, ptr %t24
  %t33 = icmp eq i32 %t32, 20
  br i1 %t33, label %L34, label %trap
L34:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}

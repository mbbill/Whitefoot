%Pair = type { {ptr, i64}, {ptr, i64} }
declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()
declare i64 @llvm.umin.i64(i64, i64)
declare {i64, i1} @llvm.uadd.with.overflow.i64(i64, i64)

define void @kernel(ptr %s) {
entry:
  %t6 = alloca i64
  %t12 = alloca i64
  %t16 = alloca i64
  %t17 = alloca i64
  %t36 = alloca i64
  %t47 = alloca i64
  %t51 = alloca i64
  %t1 = getelementptr %Pair, ptr %s, i32 0, i32 0
  %t2 = getelementptr {ptr, i64}, ptr %t1, i32 0, i32 0
  %t3 = load ptr, ptr %t2
  %t4 = getelementptr {ptr, i64}, ptr %t1, i32 0, i32 1
  %t5 = load i64, ptr %t4
  store i64 %t5, ptr %t6
  %t7 = getelementptr %Pair, ptr %s, i32 0, i32 1
  %t8 = getelementptr {ptr, i64}, ptr %t7, i32 0, i32 0
  %t9 = load ptr, ptr %t8
  %t10 = getelementptr {ptr, i64}, ptr %t7, i32 0, i32 1
  %t11 = load i64, ptr %t10
  store i64 %t11, ptr %t12
  %t13 = load i64, ptr %t6
  %t14 = load i64, ptr %t12
  %t15 = call i64 @llvm.umin.i64(i64 %t13, i64 %t14)
  store i64 %t15, ptr %t16
  store i64 0, ptr %t17
  br label %L18
L18:
  %t20 = load i64, ptr %t17
  %t21 = load i64, ptr %t16
  %t22 = icmp uge i64 %t20, %t21
  br i1 %t22, label %L24, label %L25
L24:
  br label %L19
L25:
  br label %L23
L23:
  %t26 = getelementptr %Pair, ptr %s, i32 0, i32 0
  %t27 = getelementptr {ptr, i64}, ptr %t26, i32 0, i32 0
  %t28 = load ptr, ptr %t27
  %t29 = getelementptr {ptr, i64}, ptr %t26, i32 0, i32 1
  %t30 = load i64, ptr %t29
  %t31 = load i64, ptr %t17
  %t32 = icmp ult i64 %t31, %t30
  br i1 %t32, label %L33, label %trap
L33:
  %t34 = getelementptr i64, ptr %t28, i64 %t31
  %t35 = load i64, ptr %t34
  store i64 %t35, ptr %t36
  %t37 = getelementptr %Pair, ptr %s, i32 0, i32 1
  %t38 = getelementptr {ptr, i64}, ptr %t37, i32 0, i32 0
  %t39 = load ptr, ptr %t38
  %t40 = getelementptr {ptr, i64}, ptr %t37, i32 0, i32 1
  %t41 = load i64, ptr %t40
  %t42 = load i64, ptr %t17
  %t43 = icmp ult i64 %t42, %t41
  br i1 %t43, label %L44, label %trap
L44:
  %t45 = getelementptr i64, ptr %t39, i64 %t42
  %t46 = load i64, ptr %t45
  store i64 %t46, ptr %t47
  %t48 = load i64, ptr %t36
  %t49 = load i64, ptr %t47
  %t50 = add i64 %t48, %t49
  store i64 %t50, ptr %t51
  %t52 = load i64, ptr %t51
  %t53 = getelementptr %Pair, ptr %s, i32 0, i32 0
  %t54 = getelementptr {ptr, i64}, ptr %t53, i32 0, i32 0
  %t55 = load ptr, ptr %t54
  %t56 = getelementptr {ptr, i64}, ptr %t53, i32 0, i32 1
  %t57 = load i64, ptr %t56
  %t58 = load i64, ptr %t17
  %t59 = icmp ult i64 %t58, %t57
  br i1 %t59, label %L60, label %trap
L60:
  %t61 = getelementptr i64, ptr %t55, i64 %t58
  store i64 %t52, ptr %t61
  %t62 = load i64, ptr %t17
  %t63 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t62, i64 1)
  %t64 = extractvalue {i64, i1} %t63, 0
  %t65 = extractvalue {i64, i1} %t63, 1
  br i1 %t65, label %trap, label %L66
L66:
  store i64 %t64, ptr %t17
  br label %L18
L19:
  ret void
trap:
  call void @llvm.trap()
  unreachable
}

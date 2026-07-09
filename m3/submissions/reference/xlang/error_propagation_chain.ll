%Result = type { i32, i64 }
declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define %Result @step_a(i64 %x) nounwind willreturn memory(none) {
entry:
  %t1 = icmp slt i64 %x, 0
  br i1 %t1, label %L3, label %L4
L3:
  %t5 = zext i32 0 to i64
  %t6 = insertvalue %Result undef, i32 1, 0
  %t7 = insertvalue %Result %t6, i64 %t5, 1
  ret %Result %t7
L4:
  br label %L2
L2:
  %t8 = mul i64 %x, 2
  %t9 = alloca i64
  store i64 %t8, ptr %t9
  %t10 = load i64, ptr %t9
  %t11 = insertvalue %Result undef, i32 0, 0
  %t12 = insertvalue %Result %t11, i64 %t10, 1
  ret %Result %t12
}

define %Result @step_b(i64 %x) nounwind willreturn memory(none) {
entry:
  %t1 = icmp sgt i64 %x, 100
  br i1 %t1, label %L3, label %L4
L3:
  %t5 = zext i32 1 to i64
  %t6 = insertvalue %Result undef, i32 1, 0
  %t7 = insertvalue %Result %t6, i64 %t5, 1
  ret %Result %t7
L4:
  br label %L2
L2:
  %t8 = add i64 %x, 1
  %t9 = alloca i64
  store i64 %t8, ptr %t9
  %t10 = load i64, ptr %t9
  %t11 = insertvalue %Result undef, i32 0, 0
  %t12 = insertvalue %Result %t11, i64 %t10, 1
  ret %Result %t12
}

define %Result @chain(i64 %x) nounwind willreturn memory(none) {
entry:
  %t1 = call %Result @step_a(i64 %x)
  %t2 = alloca %Result
  store %Result %t1, ptr %t2
  %t4 = getelementptr %Result, ptr %t2, i32 0, i32 0
  %t5 = load i32, ptr %t4
  %t8 = icmp eq i32 %t5, 0
  br i1 %t8, label %L6, label %L7
L6:
  %t9 = getelementptr %Result, ptr %t2, i32 0, i32 1
  %t10 = load i64, ptr %t9
  br label %L3
L7:
  %t13 = icmp eq i32 %t5, 1
  br i1 %t13, label %L11, label %L12
L11:
  %t14 = getelementptr %Result, ptr %t2, i32 0, i32 1
  %t15 = load i64, ptr %t14
  %t16 = insertvalue %Result undef, i32 1, 0
  %t17 = insertvalue %Result %t16, i64 %t15, 1
  ret %Result %t17
L12:
  unreachable
L3:
  %t18 = call %Result @step_b(i64 %t10)
  %t19 = alloca %Result
  store %Result %t18, ptr %t19
  %t21 = getelementptr %Result, ptr %t19, i32 0, i32 0
  %t22 = load i32, ptr %t21
  %t25 = icmp eq i32 %t22, 0
  br i1 %t25, label %L23, label %L24
L23:
  %t26 = getelementptr %Result, ptr %t19, i32 0, i32 1
  %t27 = load i64, ptr %t26
  br label %L20
L24:
  %t30 = icmp eq i32 %t22, 1
  br i1 %t30, label %L28, label %L29
L28:
  %t31 = getelementptr %Result, ptr %t19, i32 0, i32 1
  %t32 = load i64, ptr %t31
  %t33 = insertvalue %Result undef, i32 1, 0
  %t34 = insertvalue %Result %t33, i64 %t32, 1
  ret %Result %t34
L29:
  unreachable
L20:
  %t35 = insertvalue %Result undef, i32 0, 0
  %t36 = insertvalue %Result %t35, i64 %t27, 1
  ret %Result %t36
}

define i32 @main() nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = call %Result @chain(i64 20)
  %t2 = alloca %Result
  store %Result %t1, ptr %t2
  %t4 = getelementptr %Result, ptr %t2, i32 0, i32 0
  %t5 = load i32, ptr %t4
  %t8 = icmp eq i32 %t5, 0
  br i1 %t8, label %L6, label %L7
L6:
  %t9 = getelementptr %Result, ptr %t2, i32 0, i32 1
  %t10 = load i64, ptr %t9
  %t11 = icmp eq i64 %t10, 41
  br i1 %t11, label %L12, label %trap
L12:
  br label %L3
L7:
  %t15 = icmp eq i32 %t5, 1
  br i1 %t15, label %L13, label %L14
L13:
  %t16 = getelementptr %Result, ptr %t2, i32 0, i32 1
  %t17 = load i64, ptr %t16
  %t18 = icmp eq i64 0, 1
  br i1 %t18, label %L19, label %trap
L19:
  br label %L3
L14:
  unreachable
L3:
  %t20 = call %Result @chain(i64 -1)
  %t21 = alloca %Result
  store %Result %t20, ptr %t21
  %t23 = getelementptr %Result, ptr %t21, i32 0, i32 0
  %t24 = load i32, ptr %t23
  %t27 = icmp eq i32 %t24, 0
  br i1 %t27, label %L25, label %L26
L25:
  %t28 = getelementptr %Result, ptr %t21, i32 0, i32 1
  %t29 = load i64, ptr %t28
  %t30 = icmp eq i64 0, 1
  br i1 %t30, label %L31, label %trap
L31:
  br label %L22
L26:
  %t34 = icmp eq i32 %t24, 1
  br i1 %t34, label %L32, label %L33
L32:
  %t35 = getelementptr %Result, ptr %t21, i32 0, i32 1
  %t36 = load i64, ptr %t35
  %t37 = icmp eq i64 %t36, 0
  br i1 %t37, label %L38, label %trap
L38:
  br label %L22
L33:
  unreachable
L22:
  %t39 = call %Result @chain(i64 60)
  %t40 = alloca %Result
  store %Result %t39, ptr %t40
  %t42 = getelementptr %Result, ptr %t40, i32 0, i32 0
  %t43 = load i32, ptr %t42
  %t46 = icmp eq i32 %t43, 0
  br i1 %t46, label %L44, label %L45
L44:
  %t47 = getelementptr %Result, ptr %t40, i32 0, i32 1
  %t48 = load i64, ptr %t47
  %t49 = icmp eq i64 0, 1
  br i1 %t49, label %L50, label %trap
L50:
  br label %L41
L45:
  %t53 = icmp eq i32 %t43, 1
  br i1 %t53, label %L51, label %L52
L51:
  %t54 = getelementptr %Result, ptr %t40, i32 0, i32 1
  %t55 = load i64, ptr %t54
  %t56 = icmp eq i64 %t55, 1
  br i1 %t56, label %L57, label %trap
L57:
  br label %L41
L52:
  unreachable
L41:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}

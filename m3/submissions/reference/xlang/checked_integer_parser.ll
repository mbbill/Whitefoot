%Result = type { i32, i64 }
declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()
declare ptr @malloc(i64)
declare {i64, i1} @llvm.uadd.with.overflow.i64(i64, i64)
declare {i64, i1} @llvm.umul.with.overflow.i64(i64, i64)

define %Result @parse_u64({ptr, i64} %b) nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = extractvalue {ptr, i64} %b, 0
  %t2 = extractvalue {ptr, i64} %b, 1
  %t3 = alloca i64
  store i64 %t2, ptr %t3
  %t4 = load i64, ptr %t3
  %t5 = icmp eq i64 %t4, 0
  br i1 %t5, label %L7, label %L8
L7:
  %t9 = zext i32 0 to i64
  %t10 = insertvalue %Result undef, i32 1, 0
  %t11 = insertvalue %Result %t10, i64 %t9, 1
  ret %Result %t11
L8:
  br label %L6
L6:
  %t12 = alloca i64
  store i64 0, ptr %t12
  %t13 = alloca i64
  store i64 0, ptr %t13
  br label %L14
L14:
  %t16 = load i64, ptr %t13
  %t17 = load i64, ptr %t3
  %t18 = icmp uge i64 %t16, %t17
  br i1 %t18, label %L20, label %L21
L20:
  br label %L15
L21:
  br label %L19
L19:
  %t22 = load i64, ptr %t13
  %t23 = icmp ult i64 %t22, %t2
  br i1 %t23, label %L24, label %trap
L24:
  %t25 = getelementptr i8, ptr %t1, i64 %t22
  %t26 = load i8, ptr %t25
  %t27 = alloca i8
  store i8 %t26, ptr %t27
  %t28 = load i8, ptr %t27
  %t29 = icmp ult i8 %t28, 48
  br i1 %t29, label %L31, label %L32
L31:
  %t33 = zext i32 1 to i64
  %t34 = insertvalue %Result undef, i32 1, 0
  %t35 = insertvalue %Result %t34, i64 %t33, 1
  ret %Result %t35
L32:
  br label %L30
L30:
  %t36 = load i8, ptr %t27
  %t37 = icmp ugt i8 %t36, 57
  br i1 %t37, label %L39, label %L40
L39:
  %t41 = zext i32 1 to i64
  %t42 = insertvalue %Result undef, i32 1, 0
  %t43 = insertvalue %Result %t42, i64 %t41, 1
  ret %Result %t43
L40:
  br label %L38
L38:
  %t44 = load i8, ptr %t27
  %t45 = zext i8 %t44 to i64
  %t46 = alloca i64
  store i64 %t45, ptr %t46
  %t47 = load i64, ptr %t46
  %t48 = sub i64 %t47, 48
  %t49 = alloca i64
  store i64 %t48, ptr %t49
  %t50 = alloca i64
  %t51 = load i64, ptr %t12
  %t52 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 %t51, i64 10)
  %t53 = extractvalue {i64, i1} %t52, 0
  %t54 = extractvalue {i64, i1} %t52, 1
  %t58 = icmp eq i1 %t54, 0
  br i1 %t58, label %L56, label %L57
L56:
  store i64 %t53, ptr %t50
  br label %L55
L57:
  %t61 = icmp eq i1 %t54, 1
  br i1 %t61, label %L59, label %L60
L59:
  %t62 = zext i32 2 to i64
  %t63 = insertvalue %Result undef, i32 1, 0
  %t64 = insertvalue %Result %t63, i64 %t62, 1
  ret %Result %t64
L60:
  unreachable
L55:
  %t65 = alloca i64
  %t66 = load i64, ptr %t50
  %t67 = load i64, ptr %t49
  %t68 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t66, i64 %t67)
  %t69 = extractvalue {i64, i1} %t68, 0
  %t70 = extractvalue {i64, i1} %t68, 1
  %t74 = icmp eq i1 %t70, 0
  br i1 %t74, label %L72, label %L73
L72:
  store i64 %t69, ptr %t65
  br label %L71
L73:
  %t77 = icmp eq i1 %t70, 1
  br i1 %t77, label %L75, label %L76
L75:
  %t78 = zext i32 2 to i64
  %t79 = insertvalue %Result undef, i32 1, 0
  %t80 = insertvalue %Result %t79, i64 %t78, 1
  ret %Result %t80
L76:
  unreachable
L71:
  %t81 = load i64, ptr %t65
  store i64 %t81, ptr %t12
  %t82 = load i64, ptr %t13
  %t83 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t82, i64 1)
  %t84 = extractvalue {i64, i1} %t83, 0
  %t85 = extractvalue {i64, i1} %t83, 1
  br i1 %t85, label %trap, label %L86
L86:
  store i64 %t84, ptr %t13
  br label %L14
L15:
  %t87 = load i64, ptr %t12
  %t88 = insertvalue %Result undef, i32 0, 0
  %t89 = insertvalue %Result %t88, i64 %t87, 1
  ret %Result %t89
trap:
  call void @llvm.trap()
  unreachable
}

define void @expect_ok(%Result %r, i64 %want) nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = alloca %Result
  store %Result %r, ptr %t1
  %t3 = getelementptr %Result, ptr %t1, i32 0, i32 0
  %t4 = load i32, ptr %t3
  %t7 = icmp eq i32 %t4, 0
  br i1 %t7, label %L5, label %L6
L5:
  %t8 = getelementptr %Result, ptr %t1, i32 0, i32 1
  %t9 = load i64, ptr %t8
  %t10 = icmp eq i64 %t9, %want
  br i1 %t10, label %L11, label %trap
L11:
  br label %L2
L6:
  %t14 = icmp eq i32 %t4, 1
  br i1 %t14, label %L12, label %L13
L12:
  %t15 = getelementptr %Result, ptr %t1, i32 0, i32 1
  %t16 = load i64, ptr %t15
  %t17 = icmp eq i64 0, 1
  br i1 %t17, label %L18, label %trap
L18:
  br label %L2
L13:
  unreachable
L2:
  ret void
trap:
  call void @llvm.trap()
  unreachable
}

define void @expect_err(%Result %r, i64 %want_tag) nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = alloca %Result
  store %Result %r, ptr %t1
  %t3 = getelementptr %Result, ptr %t1, i32 0, i32 0
  %t4 = load i32, ptr %t3
  %t7 = icmp eq i32 %t4, 0
  br i1 %t7, label %L5, label %L6
L5:
  %t8 = getelementptr %Result, ptr %t1, i32 0, i32 1
  %t9 = load i64, ptr %t8
  %t10 = icmp eq i64 0, 1
  br i1 %t10, label %L11, label %trap
L11:
  br label %L2
L6:
  %t14 = icmp eq i32 %t4, 1
  br i1 %t14, label %L12, label %L13
L12:
  %t15 = getelementptr %Result, ptr %t1, i32 0, i32 1
  %t16 = load i64, ptr %t15
  %t17 = icmp eq i64 %t16, %want_tag
  br i1 %t17, label %L18, label %trap
L18:
  br label %L2
L13:
  unreachable
L2:
  ret void
trap:
  call void @llvm.trap()
  unreachable
}

define {ptr, i64} @digits20(i8 %d0, i8 %d1, i8 %d2, i8 %d3, i8 %d4, i8 %d5, i8 %d6, i8 %d7, i8 %d8, i8 %d9, i8 %d10, i8 %d11, i8 %d12, i8 %d13, i8 %d14, i8 %d15, i8 %d16, i8 %d17, i8 %d18, i8 %d19) nounwind {
entry:
  %t1 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 20, i64 1)
  %t2 = extractvalue {i64, i1} %t1, 0
  %t3 = extractvalue {i64, i1} %t1, 1
  br i1 %t3, label %trap, label %L4
L4:
  %t5 = call ptr @malloc(i64 %t2)
  %t6 = alloca i64
  store i64 0, ptr %t6
  br label %L7
L7:
  %t10 = load i64, ptr %t6
  %t11 = icmp ult i64 %t10, 20
  br i1 %t11, label %L8, label %L9
L8:
  %t12 = getelementptr i8, ptr %t5, i64 %t10
  store i8 0, ptr %t12
  %t13 = add i64 %t10, 1
  store i64 %t13, ptr %t6
  br label %L7
L9:
  %t14 = icmp ult i64 0, 20
  br i1 %t14, label %L15, label %trap
L15:
  %t16 = getelementptr i8, ptr %t5, i64 0
  store i8 %d0, ptr %t16
  %t17 = icmp ult i64 1, 20
  br i1 %t17, label %L18, label %trap
L18:
  %t19 = getelementptr i8, ptr %t5, i64 1
  store i8 %d1, ptr %t19
  %t20 = icmp ult i64 2, 20
  br i1 %t20, label %L21, label %trap
L21:
  %t22 = getelementptr i8, ptr %t5, i64 2
  store i8 %d2, ptr %t22
  %t23 = icmp ult i64 3, 20
  br i1 %t23, label %L24, label %trap
L24:
  %t25 = getelementptr i8, ptr %t5, i64 3
  store i8 %d3, ptr %t25
  %t26 = icmp ult i64 4, 20
  br i1 %t26, label %L27, label %trap
L27:
  %t28 = getelementptr i8, ptr %t5, i64 4
  store i8 %d4, ptr %t28
  %t29 = icmp ult i64 5, 20
  br i1 %t29, label %L30, label %trap
L30:
  %t31 = getelementptr i8, ptr %t5, i64 5
  store i8 %d5, ptr %t31
  %t32 = icmp ult i64 6, 20
  br i1 %t32, label %L33, label %trap
L33:
  %t34 = getelementptr i8, ptr %t5, i64 6
  store i8 %d6, ptr %t34
  %t35 = icmp ult i64 7, 20
  br i1 %t35, label %L36, label %trap
L36:
  %t37 = getelementptr i8, ptr %t5, i64 7
  store i8 %d7, ptr %t37
  %t38 = icmp ult i64 8, 20
  br i1 %t38, label %L39, label %trap
L39:
  %t40 = getelementptr i8, ptr %t5, i64 8
  store i8 %d8, ptr %t40
  %t41 = icmp ult i64 9, 20
  br i1 %t41, label %L42, label %trap
L42:
  %t43 = getelementptr i8, ptr %t5, i64 9
  store i8 %d9, ptr %t43
  %t44 = icmp ult i64 10, 20
  br i1 %t44, label %L45, label %trap
L45:
  %t46 = getelementptr i8, ptr %t5, i64 10
  store i8 %d10, ptr %t46
  %t47 = icmp ult i64 11, 20
  br i1 %t47, label %L48, label %trap
L48:
  %t49 = getelementptr i8, ptr %t5, i64 11
  store i8 %d11, ptr %t49
  %t50 = icmp ult i64 12, 20
  br i1 %t50, label %L51, label %trap
L51:
  %t52 = getelementptr i8, ptr %t5, i64 12
  store i8 %d12, ptr %t52
  %t53 = icmp ult i64 13, 20
  br i1 %t53, label %L54, label %trap
L54:
  %t55 = getelementptr i8, ptr %t5, i64 13
  store i8 %d13, ptr %t55
  %t56 = icmp ult i64 14, 20
  br i1 %t56, label %L57, label %trap
L57:
  %t58 = getelementptr i8, ptr %t5, i64 14
  store i8 %d14, ptr %t58
  %t59 = icmp ult i64 15, 20
  br i1 %t59, label %L60, label %trap
L60:
  %t61 = getelementptr i8, ptr %t5, i64 15
  store i8 %d15, ptr %t61
  %t62 = icmp ult i64 16, 20
  br i1 %t62, label %L63, label %trap
L63:
  %t64 = getelementptr i8, ptr %t5, i64 16
  store i8 %d16, ptr %t64
  %t65 = icmp ult i64 17, 20
  br i1 %t65, label %L66, label %trap
L66:
  %t67 = getelementptr i8, ptr %t5, i64 17
  store i8 %d17, ptr %t67
  %t68 = icmp ult i64 18, 20
  br i1 %t68, label %L69, label %trap
L69:
  %t70 = getelementptr i8, ptr %t5, i64 18
  store i8 %d18, ptr %t70
  %t71 = icmp ult i64 19, 20
  br i1 %t71, label %L72, label %trap
L72:
  %t73 = getelementptr i8, ptr %t5, i64 19
  store i8 %d19, ptr %t73
  %t74 = insertvalue {ptr, i64} undef, ptr %t5, 0
  %t75 = insertvalue {ptr, i64} %t74, i64 20, 1
  ret {ptr, i64} %t75
trap:
  call void @llvm.trap()
  unreachable
}

define i32 @main() nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 1, i64 1)
  %t2 = extractvalue {i64, i1} %t1, 0
  %t3 = extractvalue {i64, i1} %t1, 1
  br i1 %t3, label %trap, label %L4
L4:
  %t5 = call ptr @malloc(i64 %t2)
  %t6 = alloca i64
  store i64 0, ptr %t6
  br label %L7
L7:
  %t10 = load i64, ptr %t6
  %t11 = icmp ult i64 %t10, 1
  br i1 %t11, label %L8, label %L9
L8:
  %t12 = getelementptr i8, ptr %t5, i64 %t10
  store i8 48, ptr %t12
  %t13 = add i64 %t10, 1
  store i64 %t13, ptr %t6
  br label %L7
L9:
  %t14 = insertvalue {ptr, i64} undef, ptr %t5, 0
  %t15 = insertvalue {ptr, i64} %t14, i64 1, 1
  %t16 = call %Result @parse_u64({ptr, i64} %t15)
  %t17 = alloca %Result
  store %Result %t16, ptr %t17
  %t18 = load %Result, ptr %t17
  call void @expect_ok(%Result %t18, i64 0)
  %t19 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 2, i64 1)
  %t20 = extractvalue {i64, i1} %t19, 0
  %t21 = extractvalue {i64, i1} %t19, 1
  br i1 %t21, label %trap, label %L22
L22:
  %t23 = call ptr @malloc(i64 %t20)
  %t24 = alloca i64
  store i64 0, ptr %t24
  br label %L25
L25:
  %t28 = load i64, ptr %t24
  %t29 = icmp ult i64 %t28, 2
  br i1 %t29, label %L26, label %L27
L26:
  %t30 = getelementptr i8, ptr %t23, i64 %t28
  store i8 48, ptr %t30
  %t31 = add i64 %t28, 1
  store i64 %t31, ptr %t24
  br label %L25
L27:
  %t32 = icmp ult i64 0, 2
  br i1 %t32, label %L33, label %trap
L33:
  %t34 = getelementptr i8, ptr %t23, i64 0
  store i8 52, ptr %t34
  %t35 = icmp ult i64 1, 2
  br i1 %t35, label %L36, label %trap
L36:
  %t37 = getelementptr i8, ptr %t23, i64 1
  store i8 50, ptr %t37
  %t38 = insertvalue {ptr, i64} undef, ptr %t23, 0
  %t39 = insertvalue {ptr, i64} %t38, i64 2, 1
  %t40 = call %Result @parse_u64({ptr, i64} %t39)
  %t41 = alloca %Result
  store %Result %t40, ptr %t41
  %t42 = load %Result, ptr %t41
  call void @expect_ok(%Result %t42, i64 42)
  %t43 = call {ptr, i64} @digits20(i8 49, i8 56, i8 52, i8 52, i8 54, i8 55, i8 52, i8 52, i8 48, i8 55, i8 51, i8 55, i8 48, i8 57, i8 53, i8 53, i8 49, i8 54, i8 49, i8 53)
  %t44 = extractvalue {ptr, i64} %t43, 0
  %t45 = extractvalue {ptr, i64} %t43, 1
  %t46 = insertvalue {ptr, i64} undef, ptr %t44, 0
  %t47 = insertvalue {ptr, i64} %t46, i64 %t45, 1
  %t48 = call %Result @parse_u64({ptr, i64} %t47)
  %t49 = alloca %Result
  store %Result %t48, ptr %t49
  %t50 = load %Result, ptr %t49
  call void @expect_ok(%Result %t50, i64 18446744073709551615)
  %t51 = call {ptr, i64} @digits20(i8 49, i8 56, i8 52, i8 52, i8 54, i8 55, i8 52, i8 52, i8 48, i8 55, i8 51, i8 55, i8 48, i8 57, i8 53, i8 53, i8 49, i8 54, i8 49, i8 54)
  %t52 = extractvalue {ptr, i64} %t51, 0
  %t53 = extractvalue {ptr, i64} %t51, 1
  %t54 = insertvalue {ptr, i64} undef, ptr %t52, 0
  %t55 = insertvalue {ptr, i64} %t54, i64 %t53, 1
  %t56 = call %Result @parse_u64({ptr, i64} %t55)
  %t57 = alloca %Result
  store %Result %t56, ptr %t57
  %t58 = load %Result, ptr %t57
  call void @expect_err(%Result %t58, i64 2)
  %t59 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 3, i64 1)
  %t60 = extractvalue {i64, i1} %t59, 0
  %t61 = extractvalue {i64, i1} %t59, 1
  br i1 %t61, label %trap, label %L62
L62:
  %t63 = call ptr @malloc(i64 %t60)
  %t64 = alloca i64
  store i64 0, ptr %t64
  br label %L65
L65:
  %t68 = load i64, ptr %t64
  %t69 = icmp ult i64 %t68, 3
  br i1 %t69, label %L66, label %L67
L66:
  %t70 = getelementptr i8, ptr %t63, i64 %t68
  store i8 49, ptr %t70
  %t71 = add i64 %t68, 1
  store i64 %t71, ptr %t64
  br label %L65
L67:
  %t72 = icmp ult i64 1, 3
  br i1 %t72, label %L73, label %trap
L73:
  %t74 = getelementptr i8, ptr %t63, i64 1
  store i8 50, ptr %t74
  %t75 = icmp ult i64 2, 3
  br i1 %t75, label %L76, label %trap
L76:
  %t77 = getelementptr i8, ptr %t63, i64 2
  store i8 120, ptr %t77
  %t78 = insertvalue {ptr, i64} undef, ptr %t63, 0
  %t79 = insertvalue {ptr, i64} %t78, i64 3, 1
  %t80 = call %Result @parse_u64({ptr, i64} %t79)
  %t81 = alloca %Result
  store %Result %t80, ptr %t81
  %t82 = load %Result, ptr %t81
  call void @expect_err(%Result %t82, i64 1)
  %t83 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 0, i64 1)
  %t84 = extractvalue {i64, i1} %t83, 0
  %t85 = extractvalue {i64, i1} %t83, 1
  br i1 %t85, label %trap, label %L86
L86:
  %t87 = call ptr @malloc(i64 %t84)
  %t88 = alloca i64
  store i64 0, ptr %t88
  br label %L89
L89:
  %t92 = load i64, ptr %t88
  %t93 = icmp ult i64 %t92, 0
  br i1 %t93, label %L90, label %L91
L90:
  %t94 = getelementptr i8, ptr %t87, i64 %t92
  store i8 48, ptr %t94
  %t95 = add i64 %t92, 1
  store i64 %t95, ptr %t88
  br label %L89
L91:
  %t96 = insertvalue {ptr, i64} undef, ptr %t87, 0
  %t97 = insertvalue {ptr, i64} %t96, i64 0, 1
  %t98 = call %Result @parse_u64({ptr, i64} %t97)
  %t99 = alloca %Result
  store %Result %t98, ptr %t99
  %t100 = load %Result, ptr %t99
  call void @expect_err(%Result %t100, i64 0)
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}

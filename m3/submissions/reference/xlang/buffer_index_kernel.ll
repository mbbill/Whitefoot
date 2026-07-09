declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()
declare ptr @malloc(i64)
declare {i64, i1} @llvm.uadd.with.overflow.i64(i64, i64)
declare {i64, i1} @llvm.umul.with.overflow.i64(i64, i64)

define i32 @main() nounwind memory(inaccessiblemem: write) {
entry:
  %t1 = alloca i64
  store i64 1024, ptr %t1
  %t2 = load i64, ptr %t1
  %t3 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 %t2, i64 8)
  %t4 = extractvalue {i64, i1} %t3, 0
  %t5 = extractvalue {i64, i1} %t3, 1
  br i1 %t5, label %trap, label %L6
L6:
  %t7 = call ptr @malloc(i64 %t4)
  %t8 = alloca i64
  store i64 0, ptr %t8
  br label %L9
L9:
  %t12 = load i64, ptr %t8
  %t13 = icmp ult i64 %t12, %t2
  br i1 %t13, label %L10, label %L11
L10:
  %t14 = getelementptr i64, ptr %t7, i64 %t12
  store i64 0, ptr %t14
  %t15 = add i64 %t12, 1
  store i64 %t15, ptr %t8
  br label %L9
L11:
  %t16 = alloca i64
  store i64 0, ptr %t16
  br label %L17
L17:
  %t19 = load i64, ptr %t16
  %t20 = load i64, ptr %t1
  %t21 = icmp uge i64 %t19, %t20
  br i1 %t21, label %L23, label %L24
L23:
  br label %L18
L24:
  br label %L22
L22:
  %t25 = load i64, ptr %t16
  %t26 = load i64, ptr %t16
  %t27 = icmp ult i64 %t26, %t2
  br i1 %t27, label %L28, label %trap
L28:
  %t29 = getelementptr i64, ptr %t7, i64 %t26
  store i64 %t25, ptr %t29
  %t30 = load i64, ptr %t16
  %t31 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t30, i64 1)
  %t32 = extractvalue {i64, i1} %t31, 0
  %t33 = extractvalue {i64, i1} %t31, 1
  br i1 %t33, label %trap, label %L34
L34:
  store i64 %t32, ptr %t16
  br label %L17
L18:
  %t35 = alloca i64
  store i64 0, ptr %t35
  %t36 = alloca i64
  store i64 0, ptr %t36
  br label %L37
L37:
  %t39 = load i64, ptr %t36
  %t40 = load i64, ptr %t1
  %t41 = icmp uge i64 %t39, %t40
  br i1 %t41, label %L43, label %L44
L43:
  br label %L38
L44:
  br label %L42
L42:
  %t45 = load i64, ptr %t36
  %t46 = icmp ult i64 %t45, %t2
  br i1 %t46, label %L47, label %trap
L47:
  %t48 = getelementptr i64, ptr %t7, i64 %t45
  %t49 = load i64, ptr %t48
  %t50 = alloca i64
  store i64 %t49, ptr %t50
  %t51 = alloca i64
  %t52 = load i64, ptr %t35
  %t53 = load i64, ptr %t50
  %t54 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t52, i64 %t53)
  %t55 = extractvalue {i64, i1} %t54, 0
  %t56 = extractvalue {i64, i1} %t54, 1
  %t60 = icmp eq i1 %t56, 0
  br i1 %t60, label %L58, label %L59
L58:
  store i64 %t55, ptr %t51
  br label %L57
L59:
  %t63 = icmp eq i1 %t56, 1
  br i1 %t63, label %L61, label %L62
L61:
  ret i32 0
L62:
  unreachable
L57:
  %t64 = load i64, ptr %t51
  store i64 %t64, ptr %t35
  %t65 = load i64, ptr %t36
  %t66 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t65, i64 1)
  %t67 = extractvalue {i64, i1} %t66, 0
  %t68 = extractvalue {i64, i1} %t66, 1
  br i1 %t68, label %trap, label %L69
L69:
  store i64 %t67, ptr %t36
  br label %L37
L38:
  %t70 = load i64, ptr %t35
  %t71 = icmp eq i64 %t70, 523776
  br i1 %t71, label %L72, label %trap
L72:
  %t73 = alloca i64
  store i64 %t2, ptr %t73
  %t74 = load i64, ptr %t73
  %t75 = load i64, ptr %t73
  %t76 = icmp ult i64 %t74, %t75
  br i1 %t76, label %L78, label %L79
L78:
  %t80 = load i64, ptr %t73
  %t81 = icmp ult i64 %t80, %t2
  br i1 %t81, label %L82, label %trap
L82:
  %t83 = getelementptr i64, ptr %t7, i64 %t80
  %t84 = load i64, ptr %t83
  %t85 = alloca i64
  store i64 %t84, ptr %t85
  %t86 = load i64, ptr %t85
  %t87 = icmp eq i64 %t86, 0
  br i1 %t87, label %L88, label %trap
L88:
  br label %L77
L79:
  br label %L77
L77:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}

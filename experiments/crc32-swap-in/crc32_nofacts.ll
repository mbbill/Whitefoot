declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()
declare ptr @malloc(i64)
declare {i64, i1} @llvm.uadd.with.overflow.i64(i64, i64)
declare {i64, i1} @llvm.umul.with.overflow.i64(i64, i64)

define {ptr, i64} @crc32_mktab() {
entry:
  %t1 = call {i64, i1} @llvm.umul.with.overflow.i64(i64 256, i64 4)
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
  %t11 = icmp ult i64 %t10, 256
  br i1 %t11, label %L8, label %L9
L8:
  %t12 = getelementptr i32, ptr %t5, i64 %t10
  store i32 0, ptr %t12
  %t13 = add i64 %t10, 1
  store i64 %t13, ptr %t6
  br label %L7
L9:
  %t14 = alloca i64
  store i64 0, ptr %t14
  %t15 = alloca i32
  store i32 0, ptr %t15
  br label %L16
L16:
  %t18 = load i64, ptr %t14
  %t19 = icmp uge i64 %t18, 256
  br i1 %t19, label %L21, label %L22
L21:
  br label %L17
L22:
  br label %L20
L20:
  %t23 = load i32, ptr %t15
  %t24 = alloca i32
  store i32 %t23, ptr %t24
  %t25 = alloca i64
  store i64 0, ptr %t25
  br label %L26
L26:
  %t28 = load i64, ptr %t25
  %t29 = icmp uge i64 %t28, 8
  br i1 %t29, label %L31, label %L32
L31:
  br label %L27
L32:
  br label %L30
L30:
  %t33 = load i32, ptr %t24
  %t34 = and i32 %t33, 1
  %t35 = alloca i32
  store i32 %t34, ptr %t35
  %t36 = load i32, ptr %t35
  %t37 = sub i32 0, %t36
  %t38 = alloca i32
  store i32 %t37, ptr %t38
  %t39 = load i32, ptr %t38
  %t40 = and i32 %t39, 3988292384
  %t41 = alloca i32
  store i32 %t40, ptr %t41
  %t42 = load i32, ptr %t24
  %t43 = and i32 1, 31
  %t44 = lshr i32 %t42, %t43
  %t45 = alloca i32
  store i32 %t44, ptr %t45
  %t46 = load i32, ptr %t45
  %t47 = load i32, ptr %t41
  %t48 = xor i32 %t46, %t47
  store i32 %t48, ptr %t24
  %t49 = load i64, ptr %t25
  %t50 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t49, i64 1)
  %t51 = extractvalue {i64, i1} %t50, 0
  %t52 = extractvalue {i64, i1} %t50, 1
  br i1 %t52, label %trap, label %L53
L53:
  store i64 %t51, ptr %t25
  br label %L26
L27:
  %t54 = load i32, ptr %t24
  %t55 = load i64, ptr %t14
  %t56 = icmp ult i64 %t55, 256
  br i1 %t56, label %L57, label %trap
L57:
  %t58 = getelementptr i32, ptr %t5, i64 %t55
  store i32 %t54, ptr %t58
  %t59 = load i64, ptr %t14
  %t60 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t59, i64 1)
  %t61 = extractvalue {i64, i1} %t60, 0
  %t62 = extractvalue {i64, i1} %t60, 1
  br i1 %t62, label %trap, label %L63
L63:
  store i64 %t61, ptr %t14
  %t64 = load i32, ptr %t15
  %t65 = add i32 %t64, 1
  store i32 %t65, ptr %t15
  br label %L16
L17:
  %t66 = insertvalue {ptr, i64} undef, ptr %t5, 0
  %t67 = insertvalue {ptr, i64} %t66, i64 256, 1
  ret {ptr, i64} %t67
trap:
  call void @llvm.trap()
  unreachable
}

define i32 @crc32_upd({ptr, i64} %tab, i32 %crc, {ptr, i64} %data) {
entry:
  %t1 = extractvalue {ptr, i64} %tab, 0
  %t2 = extractvalue {ptr, i64} %tab, 1
  %t3 = extractvalue {ptr, i64} %data, 0
  %t4 = extractvalue {ptr, i64} %data, 1
  %t5 = alloca i64
  store i64 %t4, ptr %t5
  %t6 = xor i32 %crc, 4294967295
  %t7 = alloca i32
  store i32 %t6, ptr %t7
  %t8 = alloca i64
  store i64 0, ptr %t8
  br label %L9
L9:
  %t11 = load i64, ptr %t8
  %t12 = load i64, ptr %t5
  %t13 = icmp uge i64 %t11, %t12
  br i1 %t13, label %L15, label %L16
L15:
  br label %L10
L16:
  br label %L14
L14:
  %t17 = load i64, ptr %t8
  %t18 = icmp ult i64 %t17, %t4
  br i1 %t18, label %L19, label %trap
L19:
  %t20 = getelementptr i8, ptr %t3, i64 %t17
  %t21 = load i8, ptr %t20
  %t22 = alloca i8
  store i8 %t21, ptr %t22
  %t23 = load i8, ptr %t22
  %t24 = zext i8 %t23 to i32
  %t25 = alloca i32
  store i32 %t24, ptr %t25
  %t26 = load i32, ptr %t7
  %t27 = load i32, ptr %t25
  %t28 = xor i32 %t26, %t27
  %t29 = alloca i32
  store i32 %t28, ptr %t29
  %t30 = load i32, ptr %t29
  %t31 = and i32 %t30, 255
  %t32 = alloca i32
  store i32 %t31, ptr %t32
  %t33 = load i32, ptr %t32
  %t34 = zext i32 %t33 to i64
  %t35 = alloca i64
  store i64 %t34, ptr %t35
  %t36 = load i64, ptr %t35
  %t37 = icmp ult i64 %t36, %t2
  br i1 %t37, label %L38, label %trap
L38:
  %t39 = getelementptr i32, ptr %t1, i64 %t36
  %t40 = load i32, ptr %t39
  %t41 = alloca i32
  store i32 %t40, ptr %t41
  %t42 = load i32, ptr %t7
  %t43 = and i32 8, 31
  %t44 = lshr i32 %t42, %t43
  %t45 = alloca i32
  store i32 %t44, ptr %t45
  %t46 = load i32, ptr %t41
  %t47 = load i32, ptr %t45
  %t48 = xor i32 %t46, %t47
  store i32 %t48, ptr %t7
  %t49 = load i64, ptr %t8
  %t50 = call {i64, i1} @llvm.uadd.with.overflow.i64(i64 %t49, i64 1)
  %t51 = extractvalue {i64, i1} %t50, 0
  %t52 = extractvalue {i64, i1} %t50, 1
  br i1 %t52, label %trap, label %L53
L53:
  store i64 %t51, ptr %t8
  br label %L9
L10:
  %t54 = load i32, ptr %t7
  %t55 = xor i32 %t54, 4294967295
  ret i32 %t55
trap:
  call void @llvm.trap()
  unreachable
}
